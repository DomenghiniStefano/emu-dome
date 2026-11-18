// What the first run shows of the library: S02's count per console, S06's covers.
import type { ConsoleId, LibraryGame } from '../../emu'

/** S02's tiles (board PrimoAvvio): Game Boy and Game Boy Color share one. */
const TILES: { id: string; consoles: ConsoleId[]; label: string }[] = [
  { id: 'gba', consoles: ['gba'], label: 'console.gba' },
  { id: 'nds', consoles: ['nds'], label: 'console.nds' },
  { id: '3ds', consoles: ['3ds'], label: 'console.3ds' },
  { id: 'gb', consoles: ['gb', 'gbc'], label: 'library.added.gbAndColor' },
]

export interface ConsoleTile {
  id: string
  /** The message key of its name. */
  label: string
  count: number
}

export function consoleTiles(games: Pick<LibraryGame, 'console'>[]): ConsoleTile[] {
  return TILES.map(({ id, consoles, label }) => ({
    id,
    label,
    count: games.filter(g => consoles.includes(g.console)).length,
  }))
}

/** S06: up to `max` covers of games not hidden, by title. */
export function readyCovers<G extends Pick<LibraryGame, 'title' | 'hidden'>>(
  games: G[],
  max = 3,
): G[] {
  return [...games]
    .filter(g => !g.hidden)
    .sort((a, b) => a.title.localeCompare(b.title))
    .slice(0, max)
}
