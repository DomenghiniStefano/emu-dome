// The icon a Button draws itself (its `icon` prop, Trello #104): a Lucide icon before the words, or
// alone with `label`. Its size follows the button's size, so a screen never writes an icon size
// for it: 14 on the 24px controls and the words-only sizes, 16 on the standard ones, 18 on the
// main actions and the 44px squares, 24 on the TV's. The stroke is the app's (1.75) and opaque:
// the icon takes the button's text colour (`currentColor`), and a disabled button fades as a
// whole (ButtonFade), never the icon's own stroke.
import { ButtonSize } from './variants'

export const BUTTON_ICON_STROKE = 1.75

const PX_14: ButtonSize[] = [
  ButtonSize.Small,
  ButtonSize.Compact,
  ButtonSize.IconSmall,
  ButtonSize.IconRow,
  ButtonSize.TileAction,
  ButtonSize.Toast,
  ButtonSize.Text,
  ButtonSize.LabelLink,
  ButtonSize.Bare,
]
const PX_18: ButtonSize[] = [
  ButtonSize.Large,
  ButtonSize.Menu,
  ButtonSize.MenuMain,
  ButtonSize.ToolIcon,
  ButtonSize.IconTouch,
]
const PX_24: ButtonSize[] = [ButtonSize.Tv, ButtonSize.TvTab]

/** The icon's size in px for a button of this size (ButtonSize.Default when none is given). */
export function buttonIconPx(size: ButtonSize | undefined): number {
  const s = size ?? ButtonSize.Default
  if (PX_14.includes(s)) return 14
  if (PX_18.includes(s)) return 18
  if (PX_24.includes(s)) return 24
  return 16
}
