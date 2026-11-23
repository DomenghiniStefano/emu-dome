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

/** What G02 says: why, in words, and the file or folder it is about as Rust sends them (Trello
 * #128): `file` its name, `path` where it is (a document's URI, a save folder), each shown on its
 * own line, never parsed out of a sentence. */
export interface StartProblem {
  reason: string
  file: string | null
  path: string | null
}

/** Why a game could not start: its in-game save is unreadable (E04: whose, as Rust names it,
 * and the newest backup), or anything else (G02). */
export type LaunchFailure =
  | { kind: 'saveBroken'; game: GameId; backupAt: number | null }
  | { kind: 'startError'; problem: StartProblem }

export function launchFailure(e: unknown): LaunchFailure {
  if (isErrorCode(e) && e.code === 'saveUnreadable')
    return { kind: 'saveBroken', game: e.params.game, backupAt: e.params.backupAt }
  return { kind: 'startError', problem: startProblem(e) }
}

/** G02's text for any failure: the reason, and the file or folder when the code names one. */
export function startProblem(e: unknown): StartProblem {
  const reason = errorText(e)
  if (!isErrorCode(e)) return { reason, file: null, path: null }
  if (e.code === 'romUnreadable') return { reason, file: e.params.file, path: e.params.path }
  if (e.code === 'documentNotLocal') return { reason, file: e.params.file, path: null }
  if (e.code === 'saveFolderUnavailable') return { reason, file: null, path: e.params.folder }
  return { reason, file: null, path: null }
}
