// Where the quick menu's glass panel sits and how it is cut, per layout. The shadows, the sheet's
// 20 radius and the 640 / 1040 widths are off the design system's scales (spec Decision 5): the
// literal shadows live in QuickMenu.vue as `--qm-*` variables, awaiting Cowork.
import { QuickMenuView } from './view'

/** Thick glass without its float shadow: the quick menu draws its own. */
const GLASS = 'flex flex-col border-line bg-glass-thick backdrop-blur-glass backdrop-saturate-140'

/** Desktop: centred, 640 px wide, as tall as its content. */
export const MENU_PANEL = `${GLASS} absolute top-1/2 left-1/2 max-h-[calc(100vh-32px)] w-[640px] max-w-[calc(100vw-32px)] -translate-1/2 gap-16 rounded-16 border p-20 shadow-(--qm-panel-shadow)`

/** G09 on desktop: a bar at the bottom, 1040 px, the game visible above it. */
export const BAR_PANEL = `${GLASS} absolute bottom-20 left-1/2 w-[1040px] max-w-[calc(100vw-32px)] -translate-x-1/2 gap-16 rounded-16 border shadow-(--qm-panel-shadow)`

/**
 * Phone in portrait: a bottom sheet under the game (board «Tel-MenuRapido3», 390 × 844: the top
 * is 44 px + the 3:2 game image; G08 starts higher, G09 lower).
 */
export function sheetPanel(view: QuickMenuView): string {
  const top =
    view === QuickMenuView.Slots
      ? 'top-[calc(100vw*2/3-24px)]'
      : view === QuickMenuView.Screens
        ? 'top-[calc(100vw*2/3+112px)]'
        : 'top-[calc(100vw*2/3+44px)]'
  return `${GLASS} absolute inset-x-0 bottom-0 ${top} gap-10 rounded-t-[20px] border-t px-16 pt-8 pb-18 text-lead shadow-(--qm-sheet-shadow)`
}
