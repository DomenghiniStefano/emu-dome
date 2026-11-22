// Where the quick menu's glass panel sits and how it is cut, per layout: the desktop's panel, the
// G09 bar, the phone's sheet, the phone's side panel. Its shadows (`shadow-panel`, `shadow-sheet`), the sheet's 20 radius
// and the 640 / 1040 widths are pending tokens, awaiting Cowork (spec Decision 5).
import { cva } from 'class-variance-authority'
import { NO_INSET, type InsetPx } from '@/components/game/inset'
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
  /**
   * Phone on its side (#29, the card: «landscape: 440 px panel from the right»): the sheet's
   * content in a column as tall as the window at its right edge, the game beside it; clear of
   * the status bar and, turned the other way, of the camera.
   */
  Side: 'side',
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
        'absolute top-1/2 left-1/2 max-h-[calc(100vh-var(--spacing-window-margin))] w-panel max-w-[calc(100vw-var(--spacing-window-margin))] -translate-1/2 gap-16 rounded-16 border p-20 shadow-panel tv:zoom-tv tv:max-h-[calc((100vh-var(--spacing-window-margin))/var(--zoom-tv)/var(--tv-scale))] tv:max-w-[calc((100vw-var(--spacing-window-margin))/var(--zoom-tv)/var(--tv-scale))]',
      [PanelLayout.Bar]:
        'absolute bottom-20 left-1/2 w-bar max-w-[calc(100vw-var(--spacing-window-margin))] -translate-x-1/2 gap-16 rounded-16 border shadow-panel',
      [PanelLayout.Sheet]:
        'absolute inset-x-0 bottom-0 gap-10 rounded-t-20 border-t px-16 pt-8 pb-18 text-lead shadow-sheet',
      [PanelLayout.Side]:
        'absolute inset-y-0 right-0 w-side-panel max-w-full gap-10 border-l pt-side-panel-top pr-side-panel-end pb-12 pl-16 text-lead shadow-sheet',
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
export const SIDE_PANEL = panelVariants({ layout: PanelLayout.Side })

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

/** What the quick menu keeps clear of the game: the phone's sheet (the bottom), its side panel
 * (the right), nothing (the desktop's panel over the dimmed game, the editors over the game at
 * its real size), or `bar` (G09 on desktop: ScreensBar measures its own). */
export const MenuInset = {
  Sheet: 'sheet',
  Side: 'side',
  None: 'none',
  Bar: 'bar',
} as const
export type MenuInset = (typeof MenuInset)[keyof typeof MenuInset]

/**
 * The edges of the window the phone's panel keeps clear of the game (#29), from where the panel
 * is (`rect`, CSS px) in a window of `view` size: above the sheet the game sits against it (the
 * board's game image over the sheet), beside the side panel it is centred in what is left.
 */
export function panelInset(
  kind: Exclude<MenuInset, 'bar'>,
  rect: { left: number; top: number },
  view: { width: number; height: number },
): InsetPx {
  switch (kind) {
    case MenuInset.Sheet:
      return {
        ...NO_INSET,
        bottom: Math.max(0, Math.ceil(view.height - rect.top)),
        align: 'bottom',
      }
    case MenuInset.Side:
      return { ...NO_INSET, right: Math.max(0, Math.ceil(view.width - rect.left)) }
    case MenuInset.None:
      return NO_INSET
  }
}
