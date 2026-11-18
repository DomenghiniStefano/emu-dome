// The quick menu's screens: the main one and the pages it opens (G07–G11, G16), and what each
// page shows from the settings at the Session level («in prova»).

export const QuickMenuView = {
  /** The main screen: quick saves, speed, tools, the rows to the other pages. */
  Home: 'home',
  /** G08: every quick-save slot. */
  Slots: 'slots',
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
} as const
export type QuickMenuView = (typeof QuickMenuView)[keyof typeof QuickMenuView]

/** A page with a title under the back button. */
export type SubPage = Exclude<
  QuickMenuView,
  typeof QuickMenuView.Home | typeof QuickMenuView.Restart
>

/** The title (message key) of each page. */
export const SUB_PAGE_TITLE: Record<SubPage, string> = {
  [QuickMenuView.Slots]: 'qm.more',
  [QuickMenuView.Screens]: 'qm.row.screens.name',
  [QuickMenuView.Controls]: 'qm.row.controls.name',
  [QuickMenuView.Shortcuts]: 'qm.shortcuts',
  [QuickMenuView.Speed]: 'qm.speedPage',
}

export function isSubPage(view: QuickMenuView): view is SubPage {
  return view !== QuickMenuView.Home && view !== QuickMenuView.Restart
}

/** The settings pages whose Session rows the quick menu reads. */
export const SESSION_PAGES = ['video', 'fx', 'audio', 'speed', 'saves', 'touch']

/**
 * The setting rows each page shows at the Session level (those the schema shows: `touch.topac`
 * is phone only and arrives in Phase 4).
 */
export const SESSION_ROWS: Partial<Record<QuickMenuView, string[]>> = {
  [QuickMenuView.Screens]: ['video.scale', 'video.filter', 'fx.fxpick'],
  [QuickMenuView.Controls]: ['touch.topac'],
  [QuickMenuView.Speed]: ['speed.ffval', 'speed.slow', 'audio.ff'],
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
} as const
