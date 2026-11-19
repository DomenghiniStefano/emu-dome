// Order and filters of the library grid (boards L01 and L03). Pure: the view feeds it the games.
import type { ConsoleId, LibraryGame } from '@/emu'

/** Option ids of the `library.sort` setting (crates/emudome-core/src/settings/catalog.rs). */
export const SORTS = ['last_played', 'name', 'play_time', 'added'] as const
export type Sort = (typeof SORTS)[number]
export const DEFAULT_SORT: Sort = 'last_played'

export const isSort = (v: unknown): v is Sort => SORTS.some(s => s === v)

/** L03 «Mostra»: the view's own switches, off by default. */
export const SHOW_KEYS = ['favourites', 'unplayed', 'here', 'hidden'] as const
export type ShowKey = (typeof SHOW_KEYS)[number]

/** Console chips in this order; the view shows only the consoles present. */
export const CONSOLES: ConsoleId[] = ['gb', 'gbc', 'gba', 'nds', '3ds']

/** The games the «Mostra» switches let through (before the console chip). */
export function shown(games: LibraryGame[], show: Record<ShowKey, boolean>): LibraryGame[] {
  return games.filter(
    g =>
      (show.hidden || !g.hidden) &&
      (!show.favourites || g.favourite) &&
      (!show.unplayed || g.lastPlayed === null) &&
      (!show.here || g.available),
  )
}

const byName = (a: LibraryGame, b: LibraryGame) =>
  a.title.localeCompare(b.title, undefined, { sensitivity: 'base', numeric: true })

/** Sorted copy: newest first for times and dates, A–Z for names; ties by name. */
export function sorted(games: LibraryGame[], sort: Sort): LibraryGame[] {
  const key = (g: LibraryGame): number => {
    switch (sort) {
      case 'last_played':
        return g.lastPlayed ?? -Infinity
      case 'play_time':
        return g.playTimeSecs
      case 'added':
        return g.addedAt
      case 'name':
        return 0
    }
  }
  return [...games].sort((a, b) => key(b) - key(a) || byName(a, b))
}
