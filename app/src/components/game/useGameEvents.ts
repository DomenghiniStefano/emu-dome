// What the controller combos (already run in Rust) and the core's events change in the game
// being played, and the toasts they show: saves and loads (G17 with its «Annulla»), the slot,
// screenshots, errors, the frame size, a stop.
import { useI18n } from 'vue-i18n'
import { AUTO_SLOT, emu, type EmuEvent, type HotkeyEvent } from '@/emu'
import { inputMode } from '@/lib/inputMode'
import type { GameState } from './gameState'
import type { useGameSettings } from './useGameSettings'
import type { Toasts } from '@/composables/useToasts'

export function useGameEvents(
  state: GameState,
  settings: ReturnType<typeof useGameSettings>['settings'],
  toasts: Toasts,
  game: {
    /** Closes the menu after a load from it. */
    setMenu: (open: boolean) => Promise<void>
    /** The core stopped. */
    stopped: () => void
  },
) {
  const { t } = useI18n()
  const { menuOpen, showFps, speed, turboHeld, paused, comboSlot, undoLoadSlot, gameSize } = state

  // A load asked from the quick menu: its `stateLoaded` shows G17.
  let menuLoad: number | null = null
  /** A new game: no load from the menu is pending. */
  const reset = () => (menuLoad = null)

  /** Carica in the quick menu: loads the slot and resumes at once; `stateLoaded` shows G17. */
  async function loadFromMenu(slot: number) {
    menuLoad = slot
    try {
      await emu.loadState(slot)
    } catch (e) {
      menuLoad = null
      toasts.failWith(e)
      return
    }
    await game.setMenu(false)
  }

  async function undoLoad() {
    try {
      await emu.undoLoad()
    } catch (e) {
      toasts.failWith(e)
    }
  }

  /** G17: «Caricato lo slot N» with «Annulla» for about 5 s; with a controller the undo is in
   * the quick menu instead (board TV-MenuRapido3-Annulla). */
  function loadedToast(slot: number) {
    const text = slot === AUTO_SLOT ? t('toast.autoLoaded') : t('toast.stateLoaded', { slot })
    if (!settings.value.undoAllowed) return toasts.notify(text)
    if (inputMode.value === 'controller')
      return toasts.notify(text, { note: t('toast.undoFromMenu'), ms: 5000 })
    toasts.notify(text, {
      ms: 5000,
      action: { label: t('toast.undo'), run: () => void undoLoad() },
    })
  }

  /** What a shortcut changed: a controller combo or a key (the shortcut itself ran in Rust). */
  function onHotkey(h: HotkeyEvent) {
    switch (h.kind) {
      case 'fps':
        showFps.value = !showFps.value
        return
      case 'slot':
        comboSlot.value = h.slot
        toasts.notify(t('toast.slot', { slot: h.slot }))
        return
      case 'speed':
        speed.value = h.speed
        return
      case 'turbo':
        turboHeld.value = h.held
        return
      case 'paused':
        paused.value = h.paused
    }
  }

  /** The core's events while a game runs. */
  function onEmuEvent(ev: EmuEvent) {
    // Slot 0 is the auto quick save: it works silently. In the menu a save shows its own
    // line «Salvato nello slot N · Annulla» instead of a toast.
    if (ev.type === 'stateSaved' && ev.slot !== AUTO_SLOT) {
      undoLoadSlot.value = null
      if (!menuOpen.value) toasts.notify(t('toast.stateSaved', { slot: ev.slot }))
    }
    if (ev.type === 'stateLoaded') {
      undoLoadSlot.value = ev.slot
      if (menuLoad === ev.slot) loadedToast(ev.slot)
      else if (ev.slot !== AUTO_SLOT) toasts.notify(t('toast.stateLoaded', { slot: ev.slot }))
      menuLoad = null
    }
    if (ev.type === 'screenshotSaved') toasts.notify(t('toast.screenshotSaved'))
    if (ev.type === 'loadUndone') {
      undoLoadSlot.value = null
      toasts.notify(t('toast.loadUndone'))
    }
    if (ev.type === 'error') {
      menuLoad = null
      toasts.failWith(ev.error)
    }
    if (ev.type === 'started') gameSize.value = { w: ev.width, h: ev.height }
    if (ev.type === 'stopped') game.stopped()
  }

  return { reset, loadFromMenu, undoLoad, onHotkey, onEmuEvent }
}
