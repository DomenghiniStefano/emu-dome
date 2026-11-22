// Game details › «Versioni» (#102, mockup «Versioni regionali in libreria»): the regional versions
// of the game's family, one row each: region, that version's title, play time, last played; the
// version on the library's tile marked «In libreria», the others with «Gioca questa» and «Mostra
// questa in libreria». Pure: the components turn the keys into text.
import type { GameFamily, GameRegion, LibraryGame } from '@/emu'

export interface VersionRow {
  game: LibraryGame
  /** The version on the library's tile. */
  inLibrary: boolean
}

/** The rows, in the family's order (the tile's version first); versions the library no longer
 * lists are left out. None when the game is in no family. */
export function versionRows(family: GameFamily | undefined, games: LibraryGame[]): VersionRow[] {
  if (!family) return []
  const byId = new Map(games.map(g => [g.id, g]))
  return family.versions.flatMap(id => {
    const game = byId.get(id)
    return game ? [{ game, inLibrary: id === family.shown }] : []
  })
}

/** The i18n key of a region's short tag (EU, US, JP…); `none` for a game whose code names none. */
export function regionTagKey(region: GameRegion | null): string {
  return `details.versions.regionTag.${region ?? 'none'}`
}
