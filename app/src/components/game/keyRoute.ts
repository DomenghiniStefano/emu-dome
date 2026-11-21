// Where a key goes (useGameKeys): the quick menu (Esc), fullscreen (F11) and moving the focus in
// menus stay in the interface; every other key of a game goes raw to Rust, which routes it to
// the game's buttons and the keyboard's shortcuts as it routes the controllers
// (crates/emudome-core/src/keyboard.rs). Pure; useGameKeys carries the route out.
import { keyShortcut } from '@/lib/keyboard'

/** What the app is doing when the key arrives. */
export interface KeyContext {
  /** A game is open. */
  inGame: boolean
  menuOpen: boolean
}

export type KeyRoute =
  /** Esc in the quick menu: one level up. */
  | { kind: 'back' }
  | { kind: 'openMenu' }
  /** F11: the browser's default is stopped; `toggle` on a first press only. */
  | { kind: 'fullscreen'; toggle: boolean }
  /** Outside the game or in the menu, a key pressed: arrows and Enter move the focus. */
  | { kind: 'nav' }
  /** To Rust (`emu.key`); `prevent`: the browser's default is stopped (in game, menu closed). */
  | { kind: 'rust'; prevent: boolean }
  | { kind: 'none' }

/**
 * The route of one key event. `code` is `KeyboardEvent.code`; `gameKeys` the keys of the game's
 * buttons (the keyboard's row of the remap table), only to know which keys Rust takes.
 */
export function keyRoute(
  code: string,
  pressed: boolean,
  repeat: boolean,
  ctx: KeyContext,
  gameKeys: Readonly<Record<string, unknown>>,
): KeyRoute {
  const action = keyShortcut(code)?.action
  if (action === 'menu' && pressed) return ctx.menuOpen ? { kind: 'back' } : { kind: 'openMenu' }
  if (action === 'fullscreen') return { kind: 'fullscreen', toggle: pressed && !repeat }
  const forRust = (action !== undefined && action !== 'menu') || code in gameKeys
  if (!ctx.inGame) return pressed ? { kind: 'nav' } : { kind: 'none' }
  if (ctx.menuOpen) {
    if (pressed) return { kind: 'nav' }
    // A key let go in the menu still reaches Rust: its game button and its hold end (#74).
    return forRust ? { kind: 'rust', prevent: false } : { kind: 'none' }
  }
  return forRust ? { kind: 'rust', prevent: true } : { kind: 'none' }
}
