// The Saves section's rules as pure functions (board «Salvataggi», V01, V05, E03): which games
// are listed and in what order, the search, how an imported file finds its game, and which
// sentence the empty section shows. No Vue, no bridge: unit-tested in savesModel.test.ts.
import type { LibraryGame, SavesSummary } from '@/emu'

/** Whether a game has anything saved: an in-game save, a quick save, or a broken save (E04). */
export function hasSaves(s: SavesSummary | undefined): boolean {
  return !!s && (s.inGame || s.quickCount > 0 || s.broken)
}

/** V01: the games that have saves (`summaries` by `GameId`), the most recently changed first. */
export function gamesWithSaves(
  games: readonly LibraryGame[],
  summaries: Readonly<Record<string, SavesSummary>>,
): LibraryGame[] {
  const lastChange = (g: LibraryGame) => summaries[g.id]?.lastChange ?? 0
  return games.filter(g => hasSaves(summaries[g.id])).sort((a, b) => lastChange(b) - lastChange(a))
}

/** V01's search: the games whose title holds the query, ignoring case; an empty query keeps all. */
export function filterByTitle(games: LibraryGame[], query: string): LibraryGame[] {
  const q = query.trim().toLocaleLowerCase()
  return q ? games.filter(g => g.title.toLocaleLowerCase().includes(q)) : games
}

/** The games in title order (V05's «Scegli il gioco»). */
export function sortByTitle(games: readonly LibraryGame[]): LibraryGame[] {
  return [...games].sort((a, b) => a.title.localeCompare(b.title))
}

/** A path's file name without folder and extension, lower case: what an import matches on. */
export function fileStem(path: string): string {
  return (path.split(/[\\/]/).pop() ?? path).replace(/\.[^.]+$/, '').toLocaleLowerCase()
}

/** V05: the game an imported file belongs to: the ROM with the same file name, else the same title. */
export function matchImportTarget(
  fileName: string,
  games: readonly LibraryGame[],
): LibraryGame | null {
  const stem = fileStem(fileName)
  return (
    games.find(g => fileStem(g.path) === stem) ??
    games.find(g => g.title.toLocaleLowerCase() === stem) ??
    null
  )
}

/** E03: the message key for the empty section, after the auto quick save interval (`saves.autoq`). */
export function emptyTextKey(autoq: string): string {
  if (autoq === '1min') return 'saves.empty.text1min'
  if (autoq === 'never') return 'saves.empty.textNever'
  return 'saves.empty.text5min'
}
