// The keyboard: outside the game and in the quick menu the arrows and Enter move the focus
// (nav.ts); in game the keyboard table (keyboard.ts) and the game's keys (keymap.ts).
import { onUnmounted, watch } from 'vue'
import { emu } from '@/emu'
import { report } from '@/lib/errors'
import { watchInput } from '@/lib/inputMode'
import type { KeyAction } from '@/lib/keyboard'
import { keymap, loadKeymap } from '@/lib/keymap'
import { keyRoute } from './keyRoute'
import { rewindAfter } from './session'
import { dispatchNav, keyToNav, showFocusRing } from '@/nav'
import type { GameSession } from './useGameSession'

export function useGameKeys(session: GameSession) {
  const { game, menuOpen, comboSlot, showFps, rewinding, settings } = session
  /** Tab is holding fast forward (the controller's turbo is Rust's). */
  let turboKey = false

  /** Backspace: rewind while held, or on / off at each press with `a11y.toggle`. */
  function rewindKey(pressed: boolean, repeat: boolean) {
    if (repeat) return
    session.setRewinding(rewindAfter(settings.value.rewindToggle, pressed, rewinding.value))
  }

  /**
   * The keys held now will be let go where the game cannot see it (the menu opened, the window
   * lost the focus): fast forward and a held rewind end here, the game's buttons in Rust
   * (`set_menu_open`, `release_keys`). A rewind switched on with `a11y.toggle` stays on.
   */
  function letGo() {
    if (turboKey) {
      turboKey = false
      session.setTurbo(false)
    }
    if (rewinding.value && !settings.value.rewindToggle) session.setRewinding(false)
  }
  watch(menuOpen, open => {
    if (open) letGo()
  })
  function onBlur() {
    letGo()
    if (game.value) report(emu.releaseKeys())
  }
  function onVisibility() {
    if (document.visibilityState === 'hidden') onBlur()
  }

  /** A key of the keyboard table in game, menu closed. Tab and Backspace last while held. */
  function runKey(action: KeyAction, pressed: boolean, repeat: boolean) {
    if (action === 'rewind') return rewindKey(pressed, repeat)
    if (action === 'ff_hold') {
      if (repeat) return
      turboKey = pressed
      session.setTurbo(pressed)
      return
    }
    if (!pressed || repeat) return
    switch (action) {
      case 'save':
        return report(emu.saveState(comboSlot.value))
      case 'load':
        return report(emu.loadState(comboSlot.value))
      case 'prev_slot':
      case 'next_slot':
        return void session.stepSlot(action === 'next_slot')
      case 'fps':
        showFps.value = !showFps.value
        return
      case 'screenshot':
        return void session.screenshot()
      default:
        // Esc and F11 are handled before; reserved keys never reach here (`keyShortcut`).
        return
    }
  }

  function onKey(e: KeyboardEvent, pressed: boolean) {
    if (pressed) showFocusRing()
    const route = keyRoute(
      e.code,
      pressed,
      e.repeat,
      {
        inGame: !!game.value,
        menuOpen: menuOpen.value,
        rewindToggle: settings.value.rewindToggle,
        rewinding: rewinding.value,
      },
      keymap.value,
    )
    switch (route.kind) {
      case 'endRewind':
        return rewindKey(false, false)
      case 'back':
        // In the quick menu Esc is «back»: one level up, and from its main screen the game resumes.
        return dispatchNav('back')
      case 'openMenu':
        return report(session.setMenu(true))
      case 'fullscreen':
        e.preventDefault()
        if (route.toggle) void session.toggleFullscreen()
        return
      case 'nav': {
        // Menu or shell: arrows move the focus, Enter confirms.
        const nav = keyToNav(e)
        if (nav) {
          e.preventDefault()
          dispatchNav(nav)
        }
        return
      }
      case 'shortcut':
        e.preventDefault()
        return runKey(route.action, pressed, e.repeat)
      case 'button':
        e.preventDefault()
        if (route.send) report(emu.input(route.button, pressed))
        return
      case 'none':
        return
    }
  }

  const down = (e: KeyboardEvent) => onKey(e, true)
  const up = (e: KeyboardEvent) => onKey(e, false)

  /** Starts listening, with the keys of «Tutti i giochi» until a game opens. */
  function listen() {
    window.addEventListener('keydown', down)
    window.addEventListener('keyup', up)
    window.addEventListener('blur', onBlur)
    document.addEventListener('visibilitychange', onVisibility)
    watchInput()
    void loadKeymap(false)
  }
  onUnmounted(() => {
    window.removeEventListener('keydown', down)
    window.removeEventListener('keyup', up)
    window.removeEventListener('blur', onBlur)
    document.removeEventListener('visibilitychange', onVisibility)
  })

  return { listen }
}
