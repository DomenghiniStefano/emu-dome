// Where the quick menu's glass panel sits and how it is cut, per layout: the desktop's panel, the
// G09 bar, the phone's sheet, the phone's side panel. Its shadows (`shadow-panel`, `shadow-sheet`)
// and the 640 / 1040 widths are pending tokens, awaiting a design decision (spec Decision 5).
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
   * Phone in portrait: a bottom sheet under the game, as tall as its content (owner, 2026-11-22:
   * the first level is one row of tiles and a strip, «Altro» grows the same sheet), never higher
   * than leaves `sheet-game-min` of the game above it; no handle. G09 starts at a fixed place.
   * Its foot keeps above the navigation bar should the system show it (`safe-bottom`; in game
   * the bars are hidden and it is 0, #131).
   */
  Sheet: 'sheet',
  /**
   * Phone on its side (#29; owner, 2026-11-22: 300 px, was the card's 440): the sheet's content in
   * a column as tall as the window at its right edge, the game beside it; clear of the status bar
   * and, turned the other way, of the camera.
   */
  Side: 'side',
} as const
export type PanelLayout = (typeof PanelLayout)[keyof typeof PanelLayout]

/** Where the phone's sheet starts. */
export const SheetTop = {
  /** The levels and their pages: as tall as the content. */
  Content: 'content',
  /** G09, the screens: under the game's 3:2 image and lower, more of the game above. */
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
        'absolute inset-x-0 bottom-0 max-h-[calc(100%-var(--spacing-sheet-game-min))] gap-8 rounded-t-panel border-t px-12 pt-12 pb-[calc(var(--spacing-16)+var(--spacing-safe-bottom))] text-lead shadow-sheet',
      [PanelLayout.Side]:
        'absolute inset-y-0 right-0 w-side-panel max-w-full gap-8 border-l pt-side-panel-top pr-side-panel-end pb-[calc(var(--spacing-12)+var(--spacing-safe-bottom))] pl-12 text-lead shadow-sheet',
    },
    top: {
      [SheetTop.Content]: '',
      [SheetTop.Screens]: 'top-[calc(100vw*2/3+var(--spacing-sheet-screens))]',
    },
  },
})

export const MENU_PANEL = panelVariants({ layout: PanelLayout.Menu })
export const BAR_PANEL = panelVariants({ layout: PanelLayout.Bar })
export const SIDE_PANEL = panelVariants({ layout: PanelLayout.Side })

/** The phone's sheet for a view: G09 starts at its place, every other view is as tall as it is. */
export function sheetPanel(view: QuickMenuView): string {
  const top = view === QuickMenuView.Screens ? SheetTop.Screens : SheetTop.Content
  return panelVariants({ layout: PanelLayout.Sheet, top })
}

/** What the quick menu keeps clear of the game: the phone's sheet (the bottom), its side panel
 * (the right), nothing (the desktop's panel over the dimmed game, C09 over the game at its real
 * size), or `own`: the page shown sends its own (G09 on desktop: ScreensBar; C06: TouchEditor,
 * the game placed for the touch layout being edited, #131). */
export const MenuInset = {
  Sheet: 'sheet',
  Side: 'side',
  None: 'none',
  Own: 'own',
} as const
export type MenuInset = (typeof MenuInset)[keyof typeof MenuInset]

/**
 * The edges of the window the phone's panel keeps clear of the game (#29), from where the panel
 * is (`rect`, CSS px) in a window of `view` size: above the sheet the game sits against it (the
 * board's game image over the sheet), beside the side panel it is centred in what is left.
 */
export function panelInset(
  kind: Exclude<MenuInset, 'own'>,
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
