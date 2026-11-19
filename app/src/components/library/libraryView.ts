// The Library section's pure parts (boards L01, L11, L12, E01): what the view shows, computed
// from the library and the saves. No Vue, no i18n: the components turn the keys into text.
import {
  AUTO_SLOT,
  type ConsoleId,
  type CoverKind,
  type LibraryFolder,
  type LibraryGame,
} from '@/emu'
import type { ResumePoint } from '@/lib/gameDetails'
import type { Message } from '@/i18n/message'
import { CONSOLES } from './librarySort'

/** A console chip: «Tutti» or one console present, with how many games it holds. */
export interface ConsoleChip {
  id: ConsoleId | 'all'
  count: number
}

/** «Tutti» first, then the consoles present in `games`, in the library's console order. */
export function consoleChips(games: LibraryGame[]): ConsoleChip[] {
  const present = CONSOLES.filter(c => games.some(g => g.console === c))
  return [
    { id: 'all', count: games.length },
    ...present.map(c => ({ id: c, count: games.filter(g => g.console === c).length })),
  ]
}

/** The games of one console chip. */
export function ofConsole(games: LibraryGame[], chip: ConsoleId | 'all'): LibraryGame[] {
  return chip === 'all' ? games : games.filter(g => g.console === chip)
}

/** How many games «Continua a giocare» shows. */
export const CONTINUE_COUNT = 4

/** «Continua a giocare»: the games played last, newest first, hidden ones left out. */
export function recentlyPlayed(games: LibraryGame[], count = CONTINUE_COUNT): LibraryGame[] {
  return games
    .filter(g => !g.hidden && g.lastPlayed !== null)
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))
    .slice(0, count)
}

/** E01: a folder that is not reachable, with how many of its games stay listed. */
export interface UnreachableFolder {
  path: string
  count: number
}

export function unreachableFolders(
  folders: LibraryFolder[],
  games: LibraryGame[],
): UnreachableFolder[] {
  return folders
    .filter(f => !f.reachable)
    .map(f => ({ path: f.path, count: games.filter(g => g.folder === f.path).length }))
}

/** L11: one card of «N giochi aggiunti», an i18n key and a count. */
/** Where «Riprendi» picks up; null when there is nothing. */
export function resumeSource(point: ResumePoint | undefined): Message | null {
  if (!point || point.none) return null
  if (point.slot === null) return { key: 'library.continue.fromInGame' }
  if (point.slot === AUTO_SLOT) return { key: 'library.continue.fromAuto' }
  return { key: 'library.continue.fromSlot', params: { slot: point.slot } }
}

/** L12: the covers a game can have; the ROM icon only when the ROM carries one. */
export function coverKinds(game: Pick<LibraryGame, 'hasIcon'>): CoverKind[] {
  return game.hasIcon ? ['generated', 'icon', 'user'] : ['generated', 'user']
}
