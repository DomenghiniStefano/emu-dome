// Which of the Settings section's two panes shows (Trello #29). Pure.
//
// The desk, and the phone on its side, show both: the pages on the left, the page on the right.
// The phone held upright has room for one column (like a phone's own settings app): the list of
// pages first; picking one opens it over the list, with ‹ back to it. While a search is typed the
// list keeps only its title and field, and the results show under them.

export const SettingsPane = {
  /** Both columns side by side. */
  Both: 'both',
  /** Upright: the list of pages, full width. */
  List: 'list',
  /** Upright: one page, full width, with the way back to the list. */
  Page: 'page',
  /** Upright, a search typed: the title and the field, the results under them. */
  Search: 'search',
} as const
export type SettingsPane = (typeof SettingsPane)[keyof typeof SettingsPane]

export function settingsPane(
  oneColumn: boolean,
  listOpen: boolean,
  searching: boolean,
): SettingsPane {
  if (!oneColumn) return SettingsPane.Both
  if (searching) return SettingsPane.Search
  return listOpen ? SettingsPane.List : SettingsPane.Page
}

/** Whether the list of pages (or its search field) shows. */
export const listShown = (p: SettingsPane): boolean => p !== SettingsPane.Page

/** Whether the page (or the search's results) shows. */
export const pageShown = (p: SettingsPane): boolean => p !== SettingsPane.List
