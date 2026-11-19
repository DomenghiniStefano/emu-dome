// The game being played: its state (menu, pause, speed, slots, HUD) and what the quick menu, the
// controller combos and the backend's events do to it. Opening a game is useGameLaunch's.
import { onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  AUTO_SLOT,
  emu,
  type EmuEvent,
  type GameInfo,
  type HotkeyEvent,
  type LibraryGame,
  type Speed,
  type Stats,
} from '../emu'
import { flow, gameStarted, sample } from '../gameFlow'
import { inputMode } from '../inputMode'
import { loadKeymap } from '../keymap'
import { crashReport, insetShare } from '../lib/shell/session'
import { NORMAL } from '../speeds'
import { useGameSettings } from './useGameSettings'
import type { Toasts } from './useToasts'

/** How long G14 «Chiusura» stays at least, so it can be read. */
const CLOSING_MIN_MS = 400
/** How often the stats are read while a game is open (HUD, E11 and E12 watchdogs). */
const STATS_EVERY_MS = 500

export function useGameSession(toasts: Toasts) {
  const { t } = useI18n()

  const game = ref<GameInfo | null>(null)
  // The library game being played («Colore della copertina»); null for a file.
  const playingGame = ref<LibraryGame | null>(null)
  // The game's frame size (the core's `started` event): where the native video draws it, so the
  // side background (GameSurround) leaves that rectangle clear.
  const gameSize = ref<{ w: number; h: number } | null>(null)
  const menuOpen = ref(false)
  // Paused with the controller combo (L3 + A), menu closed.
  const paused = ref(false)
  const showFps = ref(false)
  const stats = ref<Stats | null>(null)
  const speed = ref<Speed>({ ...NORMAL })
  const turboHeld = ref(false)
  const rewinding = ref(false)
  // The controller in use (quick menu).
  const controller = ref<string | null>(null)
  // The current slot (L3 + ← →, F2 / F3 move it; L3 + R1 / L1, F1 / F4 use it): the quick menu
  // opens on it. Rust holds it (`current_slot`), shared by the controller and the keyboard.
  const comboSlot = ref(1)
  // The slot of the last load, until the next save or load: «Annulla il caricamento» (`saves.undo`).
  const undoLoadSlot = ref<number | null>(null)
  // «Muto» and «A schermo intero» (quick menu tools).
  const muted = ref(false)
  const fullscreen = ref(false)
  // G09: px at the bottom the quick menu's bar covers; the game shrinks above it.
  const menuInset = ref(0)
  const { settings, load: loadSettings, concerns: settingsConcern } = useGameSettings(game, toasts)

  // A load asked from the quick menu: its `stateLoaded` shows G17.
  let menuLoad: number | null = null
  // True while closing on purpose: the core's `stopped` event is then expected.
  let closingNow = false

  // While the menu is open, Rust sends the controllers to the menu (emu://nav), not to the game.
  watch(menuOpen, open => void emu.setMenuOpen(open))
  // The keyboard's keys follow the open game's settings, or «Tutti i giochi» without one.
  watch(
    () => game.value !== null,
    inGame => void loadKeymap(inGame),
  )

  /** Whether fullscreen is on and the current slot, as Rust has them at startup. */
  async function loadDeviceState() {
    fullscreen.value = await emu.isFullscreen()
    comboSlot.value = await emu.controls.currentSlot()
  }

  /** The core runs `info`: a fresh session, then the auto quick save or the resume question. */
  async function begin(info: GameInfo, from: LibraryGame | null, resumeAuto = false) {
    playingGame.value = from
    game.value = info
    speed.value = { ...NORMAL }
    menuOpen.value = false
    paused.value = false
    rewinding.value = false
    undoLoadSlot.value = null
    menuLoad = null
    muted.value = false
    menuInset.value = 0
    gameStarted()
    void loadSettings()
    if (resumeAuto) {
      await emu.loadState(AUTO_SLOT)
    } else if (info.resumable) {
      // `saves.resume` = ask: hold the game until the answer.
      await emu.setPaused(true)
      flow.resumeAsk = true
    }
  }

  /** The game is gone (closed, or abandoned after E11): back to the shell. */
  const end = () => (game.value = null)

  /** G14: auto quick save and in-game save, then back to the section you came from. */
  async function close() {
    if (!game.value) return
    flow.closing = { title: game.value.title, autosave: game.value.autosaveOnExit }
    menuOpen.value = false
    closingNow = true
    const shown = Date.now()
    try {
      await emu.closeGame()
    } finally {
      // Long enough to read, never longer than the work.
      await new Promise(r =>
        window.setTimeout(r, Math.max(0, CLOSING_MIN_MS - (Date.now() - shown))),
      )
      closingNow = false
      flow.closing = null
      end()
    }
  }

  /** E11: the emulator stopped, or stopped answering. */
  function crashed() {
    if (!game.value || flow.crashed) return
    menuOpen.value = false
    flow.crashed = crashReport(game.value, stats.value, new Date())
  }

  async function setMenu(open: boolean) {
    if (!game.value) return
    menuOpen.value = open
    paused.value = false
    await emu.setPaused(open)
  }
  const toggleMenu = () => setMenu(!menuOpen.value)

  async function setSpeed(s: Speed) {
    speed.value = s
    await emu.setSpeed(s)
  }

  /** Salva in the quick menu (the event `stateSaved` follows). */
  const save = (slot: number) => emu.saveState(slot)

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
    await setMenu(false)
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

  async function undoLoad() {
    try {
      await emu.undoLoad()
    } catch (e) {
      toasts.failWith(e)
    }
  }

  /** G11 «Riavvia»: the game starts over and resumes. */
  async function restart() {
    await emu.reset()
    await setMenu(false)
  }

  async function setMuted(on: boolean) {
    try {
      await emu.setMuted(on)
      muted.value = on
    } catch (e) {
      toasts.failWith(e)
    }
  }

  async function toggleFullscreen() {
    fullscreen.value = await emu.toggleFullscreen()
  }

  /** G09's bar: the canvas keeps clear of it by itself, the native video is told the share. */
  function setInset(px: number) {
    menuInset.value = px
    void emu.setVideoInset(insetShare(px, window.innerHeight))
  }

  /** What a controller combo changed (the combo itself already ran in Rust). */
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

  /** F2 / F3: the previous / next slot, the same one L3 + ← → moves. */
  async function stepSlot(forward: boolean) {
    try {
      onHotkey({ kind: 'slot', slot: await emu.controls.stepSlot(forward) })
    } catch (e) {
      toasts.failWith(e)
    }
  }

  /** F12, L3 + B (in Rust) and quick menu › Strumenti: G13 «Schermata salvata» on `screenshotSaved`. */
  async function screenshot() {
    try {
      await emu.screenshot()
    } catch (e) {
      toasts.failWith(e)
    }
  }

  /** Rewind on or off (Backspace). */
  function setRewinding(on: boolean) {
    if (on === rewinding.value) return
    rewinding.value = on
    void emu.rewind(on).catch(e => toasts.failWith(e))
  }

  /** Tab held: fast forward at `speed.ffval`, back to the chosen speed when let go. */
  function setTurbo(held: boolean) {
    if (held === turboHeld.value) return
    turboHeld.value = held
    void emu.setSpeed(held ? settings.value.ffSpeed : speed.value)
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
    if (ev.type === 'stopped' && !closingNow) crashed()
  }

  /** The stats poll: the HUD's FPS, and the E11 / E12 watchdogs while the game should run. */
  let statsTimer: number | undefined
  function watchStats() {
    statsTimer = window.setInterval(async () => {
      if (!game.value) return
      const s = await emu.stats()
      stats.value = s
      const running =
        !paused.value && !menuOpen.value && !flow.resumeAsk && !flow.crashed && !flow.slow
      const seen = sample(s, running)
      if (seen === 'hang') crashed()
      else if (seen === 'slow') flow.slow = { fps: s.emuFps, target: s.targetFps }
    }, STATS_EVERY_MS)
  }
  onUnmounted(() => window.clearInterval(statsTimer))

  return {
    game,
    playingGame,
    gameSize,
    menuOpen,
    paused,
    showFps,
    stats,
    speed,
    turboHeld,
    rewinding,
    controller,
    comboSlot,
    undoLoadSlot,
    muted,
    fullscreen,
    menuInset,
    settings,
    loadSettings,
    settingsConcern,
    loadDeviceState,
    begin,
    end,
    close,
    crashed,
    setMenu,
    toggleMenu,
    setSpeed,
    save,
    loadFromMenu,
    undoLoad,
    restart,
    setMuted,
    toggleFullscreen,
    setInset,
    onHotkey,
    stepSlot,
    screenshot,
    setRewinding,
    setTurbo,
    onEmuEvent,
    watchStats,
  }
}
export type GameSession = ReturnType<typeof useGameSession>
