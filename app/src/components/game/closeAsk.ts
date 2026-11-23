// «Chiudere senza salvare?»: the quick menu's «Chiudi il gioco» asks first when the game moved
// since the user's last quick save or quick load (Rust's `Stats.unsaved`), and «Salva e chiudi»
// waits for its quick save to land before closing. Pure: useGameEnding carries it out.
import type { EmuEvent, Stats } from '@/emu'

/** The answers of the question. */
export const CloseAnswer = {
  /** A quick save in the game's quick slot, then the close flow. */
  Save: 'save',
  /** The close flow as it is (the in-game save, and the auto quick save with `saves.autoexit`). */
  Discard: 'discard',
  /** The game stays open, the quick menu too. */
  Cancel: 'cancel',
} as const
export type CloseAnswer = (typeof CloseAnswer)[keyof typeof CloseAnswer]

/**
 * Whether closing asks first: the game moved since the user's last quick save or load. Stats
 * that could not be read ask too: a question too many loses nothing, a missing one may.
 */
export function asksBeforeClose(stats: Stats | null): boolean {
  if (stats === null) return true
  return stats.running && stats.unsaved
}

/** What one of the core's events says about the quick save into `slot` being waited for. */
export function saveOutcome(ev: EmuEvent, slot: number): 'saved' | 'failed' | null {
  if (ev.type === 'stateSaved' && ev.slot === slot) return 'saved'
  if (ev.type === 'error' && ev.error.code === 'stateSaveFailed' && ev.error.params.slot === slot)
    return 'failed'
  return null
}
