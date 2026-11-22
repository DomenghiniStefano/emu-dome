// Where the quick menu's glass panel sits and how it is cut, per layout: the desktop's panel, the
// G09 bar, the phone's sheet. Its shadows (`shadow-panel`, `shadow-sheet`) and the 640 / 1040
// widths are pending tokens, awaiting Cowork (spec Decision 5).
import { cva } from 'class-variance-authority'
import { QuickMenuView } from './view'

export const PanelLayout = {
  /**
   * Desktop: centred, 640 px wide, as tall as its content. In TV mode (#30, option A) the same
   * menu scaled up as it is (`zoom-tv`: 960 px, its text from 13 to 19.5); a zoomed
   * element's viewport units grow with it, so its room in the window is divided back.
   */
  Menu: 'menu',
  /** G09 on desktop: a bar at the bottom, 1040 px, the game visible above it. */
  Bar: 'bar',
  /**
   * Phone in portrait: a bottom sheet under the game (board «Tel-MenuRapido3», 390 × 844: the top
   * is 44 px + the 3:2 game image; G08 starts higher, G09 lower).
   */
  Sheet: 'sheet',
} as const
export type PanelLayout = (typeof PanelLayout)[keyof typeof PanelLayout]

/** Where the phone's sheet starts, under the game's 3:2 image. */
export const SheetTop = {
  /** The main screen and its pages. */
  Page: 'page',
  /** G08, the slots: higher, more room for them. */
  Slots: 'slots',
  /** G09, the screens: lower, more of the game above. */
  Screens: 'screens',
} as const
export type SheetTop = (typeof SheetTop)[keyof typeof SheetTop]

/** Thick glass's material without its float shadow: the quick menu draws its own. */
export const panelVariants = cva('glass-thick-flat flex flex-col border-line', {
  variants: {
    layout: {
      [PanelLayout.Menu]:
        'absolute top-1/2 left-1/2 max-h-[calc(100vh-var(--spacing-window-margin))] w-panel max-w-[calc(100vw-var(--spacing-window-margin))] -translate-1/2 gap-12 rounded-panel border p-16 shadow-panel tv:zoom-tv tv:max-h-[calc((100vh-var(--spacing-window-margin))/var(--zoom-tv)/var(--tv-scale))] tv:max-w-[calc((100vw-var(--spacing-window-margin))/var(--zoom-tv)/var(--tv-scale))]',
      [PanelLayout.Bar]:
        'absolute bottom-16 left-1/2 w-bar max-w-[calc(100vw-var(--spacing-window-margin))] -translate-x-1/2 gap-12 rounded-panel border shadow-panel',
      [PanelLayout.Sheet]:
        'absolute inset-x-0 bottom-0 gap-10 rounded-t-panel border-t px-16 pt-8 pb-16 text-lead shadow-sheet',
    },
    top: {
      [SheetTop.Page]: 'top-[calc(100vw*2/3+var(--spacing-touch))]',
      [SheetTop.Slots]: 'top-[calc(100vw*2/3-var(--spacing-24))]',
      [SheetTop.Screens]: 'top-[calc(100vw*2/3+var(--spacing-sheet-screens))]',
    },
  },
})

export const MENU_PANEL = panelVariants({ layout: PanelLayout.Menu })
export const BAR_PANEL = panelVariants({ layout: PanelLayout.Bar })

/** The phone's sheet for a view: where it starts depends on the page. */
export function sheetPanel(view: QuickMenuView): string {
  const top =
    view === QuickMenuView.Slots
      ? SheetTop.Slots
      : view === QuickMenuView.Screens
        ? SheetTop.Screens
        : SheetTop.Page
  return panelVariants({ layout: PanelLayout.Sheet, top })
}
