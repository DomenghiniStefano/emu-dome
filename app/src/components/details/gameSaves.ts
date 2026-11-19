// The saves of one game, with as many quick save slots as the `saves.slots` setting asks for:
// the one place that reads the setting (the game details, «Continua a giocare», Saves, the quick
// menu). Reads throw; each caller reports or falls back its own way.
import { emu, type GameSaves, type LibraryGame } from '@/emu'
import { resumePoint, type ResumePoint } from '@/lib/gameDetails'
import { readSetting } from '@/lib/settingsModel'

/** The slots when the setting is missing or is not a number. */
export const DEFAULT_SLOT_COUNT = 10

/** How many quick save slots a `saves.slots` value asks for. */
export function slotCount(value: unknown): number {
  return Number(value ?? DEFAULT_SLOT_COUNT) || DEFAULT_SLOT_COUNT
}

/** `saves.slots` at «Tutti i giochi». */
export async function loadSlotCount(): Promise<number> {
  return slotCount((await readSetting('saves.slots'))?.value)
}

/** The game's saves; `slots` read from the setting when not given. */
export async function loadGameSaves(
  game: Pick<LibraryGame, 'path'>,
  slots?: number,
): Promise<GameSaves> {
  return emu.saves.game(game.path, slots ?? (await loadSlotCount()))
}

/** Nothing saved at all: the button says «Gioca». */
export const NOTHING_TO_RESUME: ResumePoint = { slot: null, thumbUrl: null, none: true }

/** Where «Riprendi» picks up for `game` (L01, L04); null when its saves cannot be read (never
 * throws). Opening the game then starts from the in-game save, as «Gioca» would. */
export async function loadResumePoint(
  game: Pick<LibraryGame, 'path'>,
  slots?: number,
): Promise<ResumePoint | null> {
  try {
    return resumePoint(await loadGameSaves(game, slots))
  } catch {
    return null
  }
}
