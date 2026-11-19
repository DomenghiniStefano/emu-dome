// The keyboard: outside the game and in the quick menu the arrows and Enter move the focus
// (nav.ts); in game the keyboard table (keyboard.ts) and the game's keys (keymap.ts).
import { onUnmounted } from 'vue'
import { emu } from '@/emu'
import { watchInput } from '@/lib/inputMode'
import { keyShortcut, type KeyAction } from '@/lib/keyboard'
import { keymap, loadKeymap } from '@/lib/keymap'
import { rewindAfter } from './session'
import { dispatchNav, keyToNav, showFocusRing } from '@/nav'
import type { GameSession } from './useGameSession'

export function useGameKeys(session: GameSession) {
  const { game, menuOpen, comboSlot, showFps, rewinding, settings } = session

  /** Backspace: rewind while held, or on / off at each press with `a11y.toggle`. */
  function rewindKey(pressed: boolean, repeat: boolean) {
    if (repeat) return
    session.setRewinding(rewindAfter(settings.value.rewindToggle, pressed, rewinding.value))
  }

  /** A key of the keyboard table in game, menu closed. Tab and Backspace last while held. */
  function runKey(action: KeyAction, pressed: boolean, repeat: boolean) {
    if (action === 'rewind') return rewindKey(pressed, repeat)
    if (action === 'ff_hold') {
      if (!repeat) session.setTurbo(pressed)
      return
    }
    if (!pressed || repeat) return
    switch (action) {
      case 'save':
        return void emu.saveState(comboSlot.value)
      case 'load':
        return void emu.loadState(comboSlot.value)
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
    const shortcut = keyShortcut(e.code)
    // Backspace let go after the menu opened: the rewind it held still ends.
    if (
      shortcut?.action === 'rewind' &&
      !pressed &&
      !settings.value.rewindToggle &&
      rewinding.value
    )
      return rewindKey(false, false)
    if (shortcut?.action === 'menu' && pressed) {
      // In the quick menu Esc is «back»: one level up, and from its main screen the game resumes.
      if (menuOpen.value) return dispatchNav('back')
      return void session.setMenu(true)
    }
    if (shortcut?.action === 'fullscreen') {
      e.preventDefault()
      if (pressed && !e.repeat) void session.toggleFullscreen()
      return
    }
    // Menu or shell: arrows move the focus, Enter confirms.
    if (!game.value || menuOpen.value) {
      const nav = pressed ? keyToNav(e) : null
      if (nav) {
        e.preventDefault()
        dispatchNav(nav)
      }
      return
    }
    if (shortcut && shortcut.action !== 'menu') {
      e.preventDefault()
      return runKey(shortcut.action, pressed, e.repeat)
    }
    const button = keymap.value[e.code]
    if (button) {
      e.preventDefault()
      if (!e.repeat) void emu.input(button, pressed)
    }
  }

  const down = (e: KeyboardEvent) => onKey(e, true)
  const up = (e: KeyboardEvent) => onKey(e, false)

  /** Starts listening, with the keys of «Tutti i giochi» until a game opens. */
  function listen() {
    window.addEventListener('keydown', down)
    window.addEventListener('keyup', up)
    watchInput()
    void loadKeymap(false)
  }
  onUnmounted(() => {
    window.removeEventListener('keydown', down)
    window.removeEventListener('keyup', up)
  })

  return { listen }
}
