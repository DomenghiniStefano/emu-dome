// The Settings section's decisions as pure functions: which consoles the scope bar offers, which
// levels a page has, the next level on L1 / R1, and which rows a page or a search shows.
// Everything comes from the Rust schema: nothing here lists settings.
import type {
  ConsoleId,
  LibraryGame,
  SettingEntry,
  SettingRow,
  SettingsSchema,
  SettingValue,
} from '@/emu'
import { cycle } from '@/lib/cycle'
import { fold, isShown, showIfHolds } from '@/lib/settingsModel'

/** The levels the Settings section edits (Session is the quick menu's). */
export type EditLevel = 'global' | 'console' | 'game'

/** Every console, in the order the scope menu lists them. */
export const CONSOLES: ConsoleId[] = ['gb', 'gbc', 'gba', 'nds', '3ds']

/** T01 «In primo piano»: not a page of the schema, the section's first screen. */
export const OVERVIEW = 'overview'

/** Pages that open with an explanation of their own (`settingsUi.intro.<page>`). */
export const INTROS = new Set(['bios', 'advanced'])

/** The values of a page at the level edited, by key. */
export type PageValues = Record<string, SettingRow>

/** The consoles of the scope menu: those with games in the library, plus the ones given. */
export function scopeConsoles(games: LibraryGame[], also: (ConsoleId | null | undefined)[]) {
  return CONSOLES.filter(c => also.includes(c) || games.some(g => g.console === c))
}

/** The console of the game played last, if any was. */
export function lastPlayedConsole(games: LibraryGame[]): ConsoleId | null {
  const last = [...games]
    .filter(g => g.lastPlayed !== null)
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))[0]
  return last?.console ?? null
}

/** The level `dir` steps away from `current`, round the list (L1 / R1); null with one level. */
export function stepLevel(levels: EditLevel[], current: EditLevel, dir: -1 | 1): EditLevel | null {
  return levels.length < 2 ? null : cycle(levels, current, dir)
}

/** The schema's entries shown on page `page` (later phases stay hidden). */
export function shownOn(schema: SettingsSchema, page: string): SettingEntry[] {
  return schema.entries.filter(e => e.page === page && isShown(e, schema))
}

/** A row's `showIf` holds with the values of its page (`rows`), else the other row's default. */
export function rowVisible(
  entry: SettingEntry,
  rows: PageValues | undefined,
  byKey: Map<string, SettingEntry>,
): boolean {
  return showIfHolds(
    entry,
    (key): SettingValue | null | undefined => rows?.[key]?.value ?? byKey.get(key)?.default,
  )
}

/** A heading and its rows, as a page or a search shows them. */
export interface ShownSection {
  id: string
  page: string
  entries: SettingEntry[]
}

/** The search's sections have this id: their heading is the page's name. */
export const RESULTS = 'results'

/**
 * The sections of `page` with the rows shown, empty sections dropped. With `onlyChangedAt`, only
 * the rows changed at that level («Mostra solo le impostazioni modificate»).
 */
export function pageSections(
  schema: SettingsSchema,
  page: string,
  values: Record<string, PageValues>,
  byKey: Map<string, SettingEntry>,
  onlyChangedAt: EditLevel | null,
): ShownSection[] {
  const rows = values[page] ?? {}
  return schema.sections
    .filter(sec => sec.page === page)
    .map(sec => ({
      id: sec.id,
      page: sec.page,
      entries: sec.keys
        .map(k => byKey.get(k))
        .filter(
          (e): e is SettingEntry =>
            !!e && isShown(e, schema) && rowVisible(e, values[e.page], byKey),
        )
        .filter(e => !onlyChangedAt || rows[e.key]?.source === onlyChangedAt),
    }))
    .filter(sec => sec.entries.length > 0)
}

/**
 * A search across every page: one section per page with hits. `words(entry)` is what an entry
 * is found by (its name and description in the current language, its key, its synonyms).
 */
export function searchSections(
  schema: SettingsSchema,
  query: string,
  values: Record<string, PageValues>,
  byKey: Map<string, SettingEntry>,
  words: (e: SettingEntry) => string,
): ShownSection[] {
  const q = fold(query.trim())
  if (!q) return []
  const out: ShownSection[] = []
  for (const page of schema.pages) {
    const hits = shownOn(schema, page).filter(
      e => rowVisible(e, values[e.page], byKey) && fold(words(e)).includes(q),
    )
    if (hits.length) out.push({ id: RESULTS, page, entries: hits })
  }
  return out
}

/** How many shown rows of `page` are set at `level` (the counters of the category list). */
export function changedCount(
  schema: SettingsSchema,
  page: string,
  rows: PageValues | undefined,
  level: EditLevel,
): number {
  if (!rows) return 0
  return shownOn(schema, page).filter(e => rows[e.key]?.source === level).length
}
