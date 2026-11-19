// Where a key goes (useGameKeys): the quick menu, fullscreen, focus navigation, a shortcut of the
// keyboard table or a game button. Pure; useGameKeys carries the route out.
import type { Button } from '@/emu'
import { keyShortcut, type KeyAction } from '@/lib/keyboard'

/** What the app is doing when the key arrives. */
export interface KeyContext {
  /** A game is open. */
  inGame: boolean
  menuOpen: boolean
  /** `a11y.toggle`: Backspace turns rewind on and off instead of holding it. */
  rewindToggle: boolean
  rewinding: boolean
}

export type KeyRoute =
  /** Backspace let go after the menu opened: the rewind it held ends. */
  | { kind: 'endRewind' }
  /** Esc in the quick menu: one level up. */
  | { kind: 'back' }
  | { kind: 'openMenu' }
  /** F11: the browser's default is stopped; `toggle` on a first press only. */
  | { kind: 'fullscreen'; toggle: boolean }
  /** Outside the game or in the menu, a key pressed: arrows and Enter move the focus. */
  | { kind: 'nav' }
  /** A key of the keyboard table in game, menu closed. */
  | { kind: 'shortcut'; action: KeyAction }
  /** A game button; `send` is false for the key's repeats. */
  | { kind: 'button'; button: Button; send: boolean }
  | { kind: 'none' }

/**
 * The route of one key event. `code` is `KeyboardEvent.code`, `keymap` the keyboard's row of the
 * remap table.
 */
export function keyRoute(
  code: string,
  pressed: boolean,
  repeat: boolean,
  ctx: KeyContext,
  keymap: Record<string, Button>,
): KeyRoute {
  const action = keyShortcut(code)?.action
  if (action === 'rewind' && !pressed && !ctx.rewindToggle && ctx.rewinding)
    return { kind: 'endRewind' }
  if (action === 'menu' && pressed) return ctx.menuOpen ? { kind: 'back' } : { kind: 'openMenu' }
  if (action === 'fullscreen') return { kind: 'fullscreen', toggle: pressed && !repeat }
  if (!ctx.inGame || ctx.menuOpen) return pressed ? { kind: 'nav' } : { kind: 'none' }
  if (action && action !== 'menu') return { kind: 'shortcut', action }
  const button = keymap[code]
  return button ? { kind: 'button', button, send: !repeat } : { kind: 'none' }
}
