// The TV library's pure parts (#30, mockup «TV-Libreria», option A): the shelves under the focused
// game, what the focused game's lines say, how many covers a shelf renders, and which cover a
// step up or down lands on. No Vue, no i18n: the components turn the keys into text.
import type { ConsoleId, GameId, LibraryGame } from '@/emu'
import type { Message } from '@/i18n/message'
import { recentlyPlayed } from '@/components/library/libraryView'
import { CONSOLES, sorted } from '@/components/library/librarySort'

/** A shelf: the games played last, the favourites, or one console's games. */
export type ShelfId = 'continue' | 'favourites' | ConsoleId

export interface Shelf {
  id: ShelfId
  games: LibraryGame[]
}

/** How many games «Continua a giocare» holds on the TV (the desk's row shows 4). */
export const TV_CONTINUE_COUNT = 12

/**
 * The shelves, top down: «Continua a giocare» (played last, newest first), «Preferiti» (A–Z),
 * then one per console in the library's console order (A–Z). Hidden games are left out, and so
 * is a shelf with no game. `tucked` (#102): the regional versions behind their family's tile,
 * left out of every shelf but «Continua a giocare», which shows the versions actually played.
 */
export function tvShelves(games: LibraryGame[], tucked: ReadonlySet<GameId> = new Set()): Shelf[] {
  const visible = games.filter(g => !g.hidden)
  const tiles = tucked.size === 0 ? visible : visible.filter(g => !tucked.has(g.id))
  const shelves: Shelf[] = [
    { id: 'continue', games: recentlyPlayed(visible, TV_CONTINUE_COUNT) },
    {
      id: 'favourites',
      games: sorted(
        tiles.filter(g => g.favourite),
        'name',
      ),
    },
    ...CONSOLES.map(c => ({
      id: c,
      games: sorted(
        tiles.filter(g => g.console === c),
        'name',
      ),
    })),
  ]
  return shelves.filter(s => s.games.length > 0)
}

/** The message key of a shelf's title. */
export function shelfTitle(id: ShelfId): string {
  switch (id) {
    case 'continue':
      return 'library.continue.title'
    case 'favourites':
      return 'tv.shelf.favourites'
    case 'gb':
    case 'gbc':
    case 'gba':
    case 'nds':
    case '3ds':
      return `console.${id}`
  }
}

/** The label over the focused game's title: «Continua a giocare» once played, else its console. */
export function heroOverline(game: Pick<LibraryGame, 'console' | 'lastPlayed'>): Message {
  return game.lastPlayed === null
    ? { key: `console.${game.console}` }
    : { key: 'library.continue.title' }
}

// ---- The covers a shelf renders
//
// A console's shelf can hold thousands of games: each renders a page of covers and one more page
// whenever the focus comes near its last rendered cover, so the next one is always there.

/** Covers rendered per page of a shelf. */
export const SHELF_PAGE = 30
/** How close to the last rendered cover the focus comes before the next page is rendered. */
const SHELF_AHEAD = 5

/** How many covers a shelf renders at first: one page, or up to `anchor` (back from its details). */
export function firstShelfLimit(shelf: Shelf, anchor: GameId | null): number {
  const at = anchor ? shelf.games.findIndex(g => g.id === anchor) : -1
  return Math.max(SHELF_PAGE, at + SHELF_AHEAD + 1)
}

/** How many covers a shelf renders once the cover at `focused` has the focus. */
export function shelfLimitAfter(limit: number, focused: number, total: number): number {
  if (limit >= total || focused < limit - SHELF_AHEAD) return limit
  return Math.min(total, limit + SHELF_PAGE)
}

/** Up or down onto another shelf: the cover whose centre is nearest `x` (index), or null. */
export function nearestCover(centres: number[], x: number): number | null {
  let best: number | null = null
  centres.forEach((c, i) => {
    const b = best === null ? undefined : centres[best]
    if (b === undefined || Math.abs(c - x) < Math.abs(b - x)) best = i
  })
  return best
}
