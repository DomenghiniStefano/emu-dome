// One opening of a game (useGameLaunch): what was opened, which file that is, and what the game
// flow shows when it cannot start. Pure; useGameLaunch holds the state and calls the bridge.
import type { GameId, LibraryGame } from '@/emu'
import { errorText, isErrorCode } from '@/i18n/labels'

/** What was opened: a file, or a library game (whose play time the backend counts). */
export type Opened = { kind: 'path'; path: string } | { kind: 'library'; game: LibraryGame }

/** One opening: what, under which title, over which cover colour. */
export interface Launch {
  opened: Opened
  title: string
  colour: string | null
}

/** Why a game could not start: its in-game save is unreadable (E04: whose, as Rust names it,
 * and the newest backup), or anything else (G02, with the reason in words). */
export type LaunchFailure =
  | { kind: 'saveBroken'; game: GameId; backupAt: number | null }
  | { kind: 'startError'; reason: string }

export function launchFailure(e: unknown): LaunchFailure {
  if (isErrorCode(e) && e.code === 'saveUnreadable')
    return { kind: 'saveBroken', game: e.params.game, backupAt: e.params.backupAt }
  return { kind: 'startError', reason: errorText(e) }
}
