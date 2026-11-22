// Where the shell is: one typed value, held by App.vue (kept while a game is open, so closing it
// comes back here) and passed down as props. It replaces the module-level «mailbox» refs a section
// used to read and clear in its setup.
//
// - `section`: the rail's current section.
// - `details`: the game whose details the Library section shows (L04–L10, #34), null = the grid.
//   Kept while another section is open: the rail's Library comes back to them, and so does closing
//   a game opened from them.
// - `arrival`: what a section is opened for, from another screen (the jumps of a game's details,
//   «Salvataggi» on a «Continua a giocare» card, back from the details onto their cover). The
//   section reads it once, when it mounts; the shell then drops it (`arrived`), so coming back to
//   the section later (after a game, say) does not repeat it.
// Pure: every move is a function of the route before it.
import type { GameId, LibraryGame, SettingKey } from '@/emu'
import { gameScopeOf, type GameFocus } from '@/lib/gameDetails'
import type { Section } from './sections'

/** What a section is opened for, once. */
export type Arrival =
  /** Back from a game's details: the grid focuses that game's cover. */
  | { section: 'library'; cover: GameId }
  /** The Saves section on one game; `importing`: its import (V05) for that game. */
  | { section: 'saves'; game: GameId; importing: boolean }
  /**
   * The Settings section at «Solo questo gioco» (L06, `game`), or on one setting's page with its
   * row lit (`row`: the search everywhere, X01).
   */
  | { section: 'settings'; game: GameFocus | null; row: SettingKey | null }
  /** L07: button remapping (C02) at «Solo questo gioco». */
  | { section: 'controls'; game: GameFocus }

export interface Route {
  section: Section
  details: GameId | null
  arrival: Arrival | null
}

export const START: Route = { section: 'library', details: null, arrival: null }

/** The arrival meant for `section`, if the route has one. */
export function arrivalFor<S extends Arrival['section']>(
  r: Route,
  section: S,
): Extract<Arrival, { section: S }> | null {
  const a = r.arrival
  return a && a.section === section ? (a as Extract<Arrival, { section: S }>) : null
}

/** The Library section shows a game's details now. */
export const showingDetails = (r: Route): boolean => r.section === 'library' && r.details !== null

/** A section, as it is (the details the Library keeps stay). */
export const openSection = (r: Route, section: Section): Route => ({
  ...r,
  section,
  arrival: null,
})

/** The rail: its Library while the details are shown goes back to the grid. */
export const pickSection = (r: Route, section: Section): Route =>
  section === 'library' && showingDetails(r) ? closeDetails(r) : openSection(r, section)

/** L04: a game's details, in the Library section. */
export const openDetails = (game: GameId): Route => ({
  section: 'library',
  details: game,
  arrival: null,
})

/** Back to the grid, on the game's cover. */
export const closeDetails = (r: Route): Route => ({
  section: 'library',
  details: null,
  arrival: r.details ? { section: 'library', cover: r.details } : null,
})

/** The Saves section on a game (L01 «Salvataggi», L05 «Mostra i backup»), or its import (L05). */
export const savesFor = (r: Route, game: GameId, importing = false): Route => ({
  ...r,
  section: 'saves',
  arrival: { section: 'saves', game, importing },
})

const focusOf = (g: Pick<LibraryGame, 'id' | 'console' | 'title'>): GameFocus => ({
  scope: gameScopeOf(g),
  title: g.title,
})

/** L06: the Settings section at «Solo questo gioco». */
export const settingsFor = (r: Route, g: Pick<LibraryGame, 'id' | 'console' | 'title'>): Route => ({
  ...r,
  section: 'settings',
  arrival: { section: 'settings', game: focusOf(g), row: null },
})

/** X01: the Settings section on `key`'s page, its row lit («Tutti i giochi»). */
export const settingRowFor = (r: Route, key: SettingKey): Route => ({
  ...r,
  section: 'settings',
  arrival: { section: 'settings', game: null, row: key },
})

/** L07: button remapping (C02) at «Solo questo gioco». */
export const remapFor = (r: Route, g: Pick<LibraryGame, 'id' | 'console' | 'title'>): Route => ({
  ...r,
  section: 'controls',
  arrival: { section: 'controls', game: focusOf(g) },
})

/** The section has read its arrival: the route without it (the same route when it had none). */
export const arrived = (r: Route): Route => (r.arrival ? { ...r, arrival: null } : r)
