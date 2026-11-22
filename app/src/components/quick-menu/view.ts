// The quick menu's screens: the main one and the pages it opens (G07–G11, G16), and what each
// page shows from the settings at the Session level («in prova»).
import type { SettingKey } from '@/emu'

export const QuickMenuView = {
  /** The main screen: quick saves, speed, tools, the rows to the other pages. */
  Home: 'home',
  /** G08: every quick-save slot (the phone: «Altro › Salvataggi»). */
  Slots: 'slots',
  /** The phone's second level, «Altro» (owner, 2026-11-22): the pages and the rarer actions. */
  More: 'more',
  /** G09: screens and picture (a bar at the bottom on desktop, the game visible above). */
  Screens: 'screens',
  /** G10: controls. */
  Controls: 'controls',
  /** G10: the controller shortcuts, read-only. */
  Shortcuts: 'shortcuts',
  /** G16: the speeds of fast forward and slow motion. */
  Speed: 'speed',
  /** G11: the confirmation of «Riavvia», in place of the menu. */
  Restart: 'restart',
  /** C09 from G09 «Personalizza gli schermi»: the screens editor over the game, in place of the
   * menu (`screen-editor/ScreenEditor.vue`). */
  ScreenEditor: 'screenEditor',
  /** C06 from G10 «Personalizza i controlli touch» (Trello #29): the touch controls editor over
   * the game, in place of the menu (`touch/TouchEditor.vue`). */
  TouchEditor: 'touchEditor',
} as const
export type QuickMenuView = (typeof QuickMenuView)[keyof typeof QuickMenuView]

/** A page with a title under the back button. */
export type SubPage = Exclude<
  QuickMenuView,
  | typeof QuickMenuView.Home
  | typeof QuickMenuView.Restart
  | typeof QuickMenuView.ScreenEditor
  | typeof QuickMenuView.TouchEditor
>

/** The title (message key) of each page. */
export const SUB_PAGE_TITLE: Record<SubPage, string> = {
  [QuickMenuView.Slots]: 'qm.more',
  [QuickMenuView.More]: 'qm.moreLabel',
  [QuickMenuView.Screens]: 'qm.row.screens.name',
  [QuickMenuView.Controls]: 'qm.row.controls.name',
  [QuickMenuView.Shortcuts]: 'qm.shortcuts',
  [QuickMenuView.Speed]: 'qm.speedPage',
}

export function isSubPage(view: QuickMenuView): view is SubPage {
  return (
    view !== QuickMenuView.Home &&
    view !== QuickMenuView.Restart &&
    view !== QuickMenuView.ScreenEditor &&
    view !== QuickMenuView.TouchEditor
  )
}

/** The phone's levels (owner, 2026-11-22): «Altro» goes back to the main screen, the pages it
 * opens (and «Riavvia»'s confirmation) back to «Altro»; the editors and the shortcuts keep their
 * own way back (useQuickMenuNavigation). */
export function phoneParent(view: QuickMenuView): QuickMenuView {
  return view === QuickMenuView.Home || view === QuickMenuView.More
    ? QuickMenuView.Home
    : QuickMenuView.More
}

/** The settings pages whose Session rows the quick menu reads (`screens`: the layout card). */
export const SESSION_PAGES = ['video', 'fx', 'screens', 'audio', 'speed', 'saves', 'touch']

/**
 * The setting rows each page shows at the Session level (those the schema shows: `touch.topac`
 * is phone only and arrives in Phase 4).
 */
export const SESSION_ROWS: Partial<Record<QuickMenuView, SettingKey[]>> = {
  [QuickMenuView.Screens]: ['video.scale', 'video.filter', 'fx.fxpick'],
  [QuickMenuView.Controls]: ['touch.topac'],
  [QuickMenuView.Speed]: ['speed.ffval', 'speed.slow', 'audio.ff'],
}

/** The pages that show settings «in prova». */
export const SESSION_VIEWS = Object.keys(SESSION_ROWS) as QuickMenuView[]

/** The page that shows the setting `key` «in prova» (the search opens it), or null. */
export function sessionViewOf(key: SettingKey): QuickMenuView | null {
  return SESSION_VIEWS.find(v => SESSION_ROWS[v]?.includes(key)) ?? null
}

/**
 * The attributes the menu moves the focus by. `data-sub-initial` marks a page's first control
 * (or a block holding it); the others name the controls the focus comes back to.
 */
export const FocusMark = {
  SubInitial: 'data-sub-initial',
  Back: 'data-qm-back',
  Shortcuts: 'data-shortcuts',
  ChosenSlot: 'data-qm-chosen',
  MoreSlots: 'data-qm-more',
  /** G09's «Personalizza gli schermi»: the focus comes back to it from the editor. */
  Customise: 'data-qm-customise',
  /** G10's «Personalizza i controlli touch»: the focus comes back to it from the C06 editor. */
  TouchEdit: 'data-qm-touch-edit',
  /**
   * A main-screen control that can open a page, by an identity that survives the main screen
   * being unmounted while the page shows: the focus comes back to the control with that value.
   */
  Opener: 'data-qm-opener',
} as const

/** The attribute naming a main-screen control the focus can come back to (`id` unique on it). */
export function openerMark(id: string): Record<string, string> {
  return { [FocusMark.Opener]: id }
}
