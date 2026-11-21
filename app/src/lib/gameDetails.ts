// Game details (L04–L10, #34): what several features share about one game: its «Solo questo
// gioco» scope, a section opened for it, where «Riprendi» picks up. Which game the details show,
// and the jumps from there to another section, are the shell's route (components/shell/route.ts).
import type { GameSaves, LibraryGame, SettingsScope } from '@/emu'

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
