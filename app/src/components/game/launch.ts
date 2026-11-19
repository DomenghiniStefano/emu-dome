// One opening of a game (useGameLaunch): what was opened, which file that is, and what the game
// flow shows when it cannot start. Pure; useGameLaunch holds the state and calls the bridge.
import type { LibraryGame } from '@/emu'
import { errorText, isErrorCode } from '@/i18n/labels'

/** What was opened: a file, or a library game (whose play time the backend counts). */
export type Opened = { kind: 'path'; path: string } | { kind: 'library'; game: LibraryGame }

/** One opening: what, under which title, over which cover colour. */
export interface Launch {
  opened: Opened
  title: string
  colour: string | null
}

/** The game file an opening reads: the path given, or the library game's. */
export const openedPath = (opened: Opened): string =>
  opened.kind === 'path' ? opened.path : opened.game.path

/** Why a game could not start: its in-game save is unreadable (E04, with the newest backup),
 * or anything else (G02, with the reason in words). */
export type LaunchFailure =
  { kind: 'saveBroken'; backupAt: number | null } | { kind: 'startError'; reason: string }

export function launchFailure(e: unknown): LaunchFailure {
  if (isErrorCode(e) && e.code === 'saveUnreadable')
    return { kind: 'saveBroken', backupAt: e.params.backupAt }
  return { kind: 'startError', reason: errorText(e) }
}
