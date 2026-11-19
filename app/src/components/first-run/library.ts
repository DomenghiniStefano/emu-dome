// What the first run shows of the library: S06's covers (S02's count per console is
// lib/consoleCounts.ts).
import type { LibraryGame } from '@/emu'

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
