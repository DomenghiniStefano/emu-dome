// The keyboard: outside the game and in the quick menu the arrows and Enter move the focus
// (nav.ts); in game every key but Esc and F11 goes raw to Rust (`emu.key`), which routes it to the
// game's buttons and the keyboard's shortcuts (crates/emudome-core/src/keyboard.rs) and carries
// them out as the controllers' (fast forward, rewind, slots...).
import { onUnmounted } from 'vue'
import { emu } from '@/emu'
import { report } from '@/lib/errors'
import { watchInput } from '@/lib/inputMode'
import { keymap, loadKeymap } from '@/lib/keymap'
import { keyRoute } from './keyRoute'
import { dispatchNav, keyToNav, showFocusRing } from '@/nav'
import type { GameSession } from './useGameSession'

export function useGameKeys(session: GameSession) {
  const { game, menuOpen } = session

  /** The keys held now are let go elsewhere (Alt-Tab, another window): Rust lets go of them,
   * ending the keyboard's fast forward and held rewind (the quick menu opening does the same in
   * Rust, `set_menu_open`), and pauses the game with `general.bgpause` until the focus is back. */
  function onBlur() {
    if (game.value) report(emu.windowFocus(false))
  }
  function onFocus() {
    if (game.value) report(emu.windowFocus(true))
  }
  function onVisibility() {
    if (document.visibilityState === 'hidden') onBlur()
  }

  function onKey(e: KeyboardEvent, pressed: boolean) {
    if (pressed) showFocusRing()
    const route = keyRoute(
      e.code,
      pressed,
      e.repeat,
      { inGame: !!game.value, menuOpen: menuOpen.value },
      keymap.value,
    )
    switch (route.kind) {
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
      case 'rust':
        if (route.prevent) e.preventDefault()
        return report(emu.key(e.code, pressed, e.repeat))
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
    window.addEventListener('focus', onFocus)
    document.addEventListener('visibilitychange', onVisibility)
    watchInput()
    void loadKeymap(false)
  }
  onUnmounted(() => {
    window.removeEventListener('keydown', down)
    window.removeEventListener('keyup', up)
    window.removeEventListener('blur', onBlur)
    window.removeEventListener('focus', onFocus)
    document.removeEventListener('visibilitychange', onVisibility)
  })

  return { listen }
}
