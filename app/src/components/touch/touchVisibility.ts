// When the touch controls are on screen (Trello #29), pure:
// - only on a touch screen, in game, with the quick menu and the C06 editor closed;
// - `touch.tpad` on: hidden while a controller is connected (they come back when it goes);
// - `touch.thide`: faded out after some seconds without a touch, back at the next one. Faded,
//   they still take a press, so the touch that brings them back is never lost.
import type { TouchSkin } from './skins'

export type HideAfter = 'never' | '5s' | '10s'

/** A `touch.thide` value: anything unknown is «never». */
export function hideAfter(value: unknown): HideAfter {
  return value === '5s' || value === '10s' ? value : 'never'
}

/** How long the controls stay after the last touch, ms (null: always). */
export function hideAfterMs(h: HideAfter): number | null {
  switch (h) {
    case 'never':
      return null
    case '5s':
      return 5000
    case '10s':
      return 10000
  }
}

/** The `touch.*` values the controls follow, as read for the open game. */
export interface TouchPrefs {
  /** `touch.tport` / `touch.tland`. */
  portrait: 'default' | 'custom'
  landscape: 'default' | 'custom'
  skin: TouchSkin
  /** `touch.topac`, per cent. */
  opacity: number
  /** `touch.thapt`, per cent (0: none). */
  haptics: number
  hide: HideAfter
  /** `touch.tpad`: hidden while a controller is connected. */
  hideWithPad: boolean
}

export const DEFAULT_PREFS: TouchPrefs = {
  portrait: 'default',
  landscape: 'default',
  skin: 'glass',
  opacity: 60,
  haptics: 40,
  hide: 'never',
  hideWithPad: true,
}

export interface ShownInputs {
  /** A touch screen is the main pointer. */
  touch: boolean
  inGame: boolean
  menuOpen: boolean
  /** The C06 editor is open (it draws the controls itself). */
  editing: boolean
  hideWithPad: boolean
  /** Controllers connected now. */
  controllers: number
}

/** Whether the controls are drawn at all. */
export function touchShown(i: ShownInputs): boolean {
  return i.touch && i.inGame && !i.menuOpen && !i.editing && !(i.hideWithPad && i.controllers > 0)
}
