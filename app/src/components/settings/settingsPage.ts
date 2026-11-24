// The Settings section's decisions as pure functions: which consoles the scope bar offers, which
// levels a page has, the next level on L1 / R1, and which rows a page or a search shows.
// Everything comes from the Rust schema: nothing here lists settings.
import type {
  ConsoleId,
  LibraryGame,
  SettingEntry,
  SettingRow,
  SettingsSchema,
  SettingsScope,
  SettingValue,
} from '@/emu'
import { cycle } from '@/lib/cycle'
import type { GameScope } from '@/lib/gameDetails'
import { fold, isShown, pageLevels, showIfHolds } from '@/lib/settingsModel'

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

/**
 * The consoles a page's console level is for: those of its entries set per console (the
 * catalogue's `consoles`: «Schermi di DS e 3DS» is about the two-screen ones, #176). Every
 * console on a page with none.
 */
export function pageConsoles(entries: SettingEntry[]): ConsoleId[] {
  const perConsole = entries.filter(e => e.levels.includes('console'))
  if (perConsole.length === 0) return CONSOLES
  return CONSOLES.filter(c => perConsole.some(e => e.consoles.includes(c)))
}

/**
 * The consoles of the scope menu: those with games in the library, plus the ones given, among
 * the ones the page is for (`pageConsoles`).
 */
export function scopeConsoles(
  games: LibraryGame[],
  also: (ConsoleId | null | undefined)[],
  offered: readonly ConsoleId[] = CONSOLES,
) {
  return CONSOLES.filter(
    c => offered.includes(c) && (also.includes(c) || games.some(g => g.console === c)),
  )
}

/** The console of the game played last, if any was (among `among`). */
export function lastPlayedConsole(
  games: LibraryGame[],
  among: readonly ConsoleId[] = CONSOLES,
): ConsoleId | null {
  const last = games
    .filter(g => g.lastPlayed !== null && among.includes(g.console))
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))[0]
  return last?.console ?? null
}

/**
 * The console the scope bar shows on a page: the one chosen when the page offers it, else the
 * page's console played last, else its first; null when it offers none. The choice itself is
 * kept: a GBA chosen on «Video» comes back on the next page that has it (#176).
 */
export function pageConsole(
  chosen: ConsoleId | null,
  offered: readonly ConsoleId[],
  games: LibraryGame[],
): ConsoleId | null {
  if (chosen && offered.includes(chosen)) return chosen
  return lastPlayedConsole(games, offered) ?? offered[0] ?? null
}

/** The level `dir` steps away from `current`, round the list (L1 / R1); null with one level. */
export function stepLevel(levels: EditLevel[], current: EditLevel, dir: -1 | 1): EditLevel | null {
  return levels.length < 2 ? null : cycle(levels, current, dir)
}

/** «Solo questo gioco» is offered: the section opened for a game and the page has entries it can
 * change. */
export function offersGameLevel(entries: SettingEntry[], forGame: boolean): boolean {
  return forGame && entries.some(e => e.levels.includes('game'))
}

/** The levels of a page's scope bar: «Tutti i giochi», the console's when the page has settings per
 * console and a console is chosen, the game's when `offersGameLevel`. */
export function scopeLevels(
  entries: SettingEntry[],
  console: ConsoleId | null,
  forGame: boolean,
): EditLevel[] {
  const out: EditLevel[] = ['global']
  if (pageLevels(entries).has('console') && console) out.push('console')
  if (offersGameLevel(entries, forGame)) out.push('game')
  return out
}

/** The level in effect: the one chosen, when the page has it, else «Tutti i giochi». */
export function effectiveLevel(levels: EditLevel[], chosen: EditLevel): EditLevel {
  return levels.includes(chosen) ? chosen : 'global'
}

/** The scope a level edits, with the console and the game it is for. */
export function scopeAt(
  level: EditLevel,
  console: ConsoleId | null,
  game: GameScope | null,
): SettingsScope {
  if (level === 'game' && game) return game
  if (level === 'console' && console) return { level: 'console', console }
  return { level: 'global' }
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
        .filter(e => !onlyChangedAt || changedAt(e, rows, onlyChangedAt)),
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

/** The levels the Settings section edits, weakest first. */
const EDIT_LEVELS: EditLevel[] = ['global', 'console', 'game']

/**
 * The level a row is set at while `level` is edited: `level` itself when the row accepts it,
 * else the strongest level below it that the row accepts («Tutti i giochi» for «Vibrazione»
 * while a console is edited); null when none does (a game's row at a console).
 */
export function rowLevel(entry: SettingEntry, level: EditLevel): EditLevel | null {
  const below = EDIT_LEVELS.slice(0, EDIT_LEVELS.indexOf(level) + 1).reverse()
  return below.find(l => entry.levels.includes(l)) ?? null
}

/**
 * Whether a row counts as changed while `level` is edited: its value comes from the level it is
 * set at there (`rowLevel`), per-device values included (#170).
 */
export function changedAt(entry: SettingEntry, rows: PageValues, level: EditLevel): boolean {
  const at = rowLevel(entry, level)
  return at !== null && rows[entry.key]?.source === at
}

/** How many shown rows of `page` are changed at `level` (the counters of the category list). */
export function changedCount(
  schema: SettingsSchema,
  page: string,
  rows: PageValues | undefined,
  level: EditLevel,
): number {
  if (!rows) return 0
  return shownOn(schema, page).filter(e => changedAt(e, rows, level)).length
}
