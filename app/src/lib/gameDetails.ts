// Game details (L04–L10, #34): which game the Library section shows the details of, and the
// jumps from there to another section for that one game. Kept outside the components, so closing
// a game opened from its details comes back to them, and a section opened for a game takes the
// game on mount (then clears it).
import { ref } from 'vue'
import type { GameId, GameSaves, LibraryGame, SettingsScope } from '@/emu'

/** The game whose details the Library section shows; null = the grid. */
export const detailsOf = ref<GameId | null>(null)
/** The cover the grid focuses when it comes back from the details. */
export const detailsReturn = ref<GameId | null>(null)

/** A game's «Solo questo gioco» level. */
export type GameScope = Extract<SettingsScope, { level: 'game' }>

/** A game's «Solo questo gioco» scope: its settings are named by its `GameId`, as everything else. */
export function gameScopeOf(g: Pick<LibraryGame, 'id' | 'console'>): GameScope {
  return { level: 'game', console: g.console, game: g.id }
}

/** A section opened at «Solo questo gioco» for one game. */
export interface GameFocus {
  scope: GameScope
  title: string
}
/** L06 «Apri tutte le impostazioni di questo gioco»: the Settings section takes it. */
export const settingsFocus = ref<GameFocus | null>(null)
/** L07 «Rimappa i pulsanti per questo gioco»: the Controls section (C02) takes it. */
export const controlsFocus = ref<GameFocus | null>(null)

/** Where «Riprendi» picks up (L01 «Continua a giocare», L04). */
export interface ResumePoint {
  /** The quick save to load; null = start from the in-game save. */
  slot: number | null
  thumbUrl: string | null
  /** Nothing saved at all: the button says «Gioca». */
  none: boolean
}

/** The newest quick save (the auto one or a slot), else the in-game save, else nothing. */
export function resumePoint(saves: GameSaves): ResumePoint {
  const newest = saves.quick
    .filter(q => q.savedAt !== null)
    .reduce<GameSaves['quick'][number] | null>(
      (best, q) => (best === null || (q.savedAt ?? 0) > (best.savedAt ?? 0) ? q : best),
      null,
    )
  return {
    slot: newest?.slot ?? null,
    thumbUrl: newest?.thumbUrl ?? null,
    none: !newest && !saves.inGame,
  }
}
