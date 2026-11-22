// The regional versions of one game (#102), as Rust grouped them (`emudome_core::library::group`):
// which family a game is in, and which games the library tucks behind their family's tile. Rust
// decides the shown version (pinned, else launched last, else added first); this only indexes it,
// once per change of the families, never per render. Pure.
import type { GameFamily, GameId, LibraryGame } from '@/emu'

export interface FamilyIndex {
  /** Every version's family. */
  byGame: Map<GameId, GameFamily>
  /** The versions behind a tile: in a family and not its shown one. */
  tucked: Set<GameId>
}

export function familyIndex(families: GameFamily[]): FamilyIndex {
  const byGame = new Map<GameId, GameFamily>()
  const tucked = new Set<GameId>()
  for (const f of families)
    for (const id of f.versions) {
      byGame.set(id, f)
      if (id !== f.shown) tucked.add(id)
    }
  return { byGame, tucked }
}

/** The games as tiles: one per family (its shown version), every other game as it is. */
export function oneTilePerFamily(games: LibraryGame[], index: FamilyIndex): LibraryGame[] {
  return index.tucked.size === 0 ? games : games.filter(g => !index.tucked.has(g.id))
}

/** How many versions the game's family holds; 0 for a game in none. */
export function versionCount(index: FamilyIndex, id: GameId): number {
  return index.byGame.get(id)?.versions.length ?? 0
}
