// The saves of one game as its details read them (L04 «Riprendi», L05 Salvataggi): with as many
// quick save slots as the `saves.slots` setting asks for.
import { emu, type GameSaves, type LibraryGame, type SettingRow } from '../../emu'

/** The slots when the setting is missing or is not a number. */
const DEFAULT_SLOTS = 10

/** How many quick save slots the `saves` page's rows ask for (`saves.slots`). */
export function quickSlotCount(rows: SettingRow[]): number {
  return Number(rows.find(r => r.key === 'saves.slots')?.value ?? DEFAULT_SLOTS) || DEFAULT_SLOTS
}

/** The game's saves, with the global number of quick save slots. */
export async function loadGameSaves(game: LibraryGame): Promise<GameSaves> {
  const rows = await emu.settings.page('saves', { level: 'global' })
  return emu.saves.game(game.path, quickSlotCount(rows))
}
