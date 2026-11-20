// What the close flow shows (Trello #86), from the last `emu://closing` event. The flow itself
// lives in Rust (`emudome_core::close_flow`); this only maps its progress to a view.
import type { ClosingProgress, ErrorCode } from '@/emu'

/** The question asked when saving does not end well. */
export type ClosingAsk =
  /** Saving takes longer than usual: «Wait» or «Close anyway». */
  | { kind: 'timeout' }
  /** A save could not be written (`error`): «Close anyway». */
  | { kind: 'failed'; error: ErrorCode | null }

/** What is on screen while a game closes. */
export interface ClosingView {
  /**
   * The app closing: «Saving, don't turn off the device» (also with a question), then «Saved».
   * Null otherwise: a game closing to the library shows G14 «Chiusura» instead.
   */
  status: 'saving' | 'saved' | null
  ask: ClosingAsk | null
}

const NOTHING: ClosingView = { status: null, ask: null }

export function closingView(progress: ClosingProgress | null): ClosingView {
  if (!progress || progress.phase === 'closed') return NOTHING
  const quit = progress.purpose === 'quit'
  switch (progress.phase) {
    case 'saving':
      return { status: quit ? 'saving' : null, ask: null }
    case 'saved':
      return { status: quit ? 'saved' : null, ask: null }
    case 'timeout':
      return { status: quit ? 'saving' : null, ask: { kind: 'timeout' } }
    case 'failed':
      return { status: quit ? 'saving' : null, ask: { kind: 'failed', error: progress.error } }
  }
}
