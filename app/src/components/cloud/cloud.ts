// The Cloud section's decisions as pure functions (Trello #31): the status card's pill and
// headline, the Drive space bar, a game's action in «Giochi su Google Drive», how long a game has
// been open elsewhere (G05), how far a transfer is. Everything comes from the backend's
// `CloudStatus` / `CloudGame` / `CloudProgress`: nothing here decides what is synced.
import type {
  CloudGame,
  CloudGameState,
  CloudProgress,
  CloudStatus,
  ErrorCode,
  GameId,
} from '@/emu'

/** The status card's state, most pressing first. */
export type StatusPill =
  | { kind: 'syncing' }
  | { kind: 'error'; error: ErrorCode }
  | { kind: 'conflicts'; n: number }
  | { kind: 'pending'; n: number }
  | { kind: 'synced' }
  | { kind: 'never' }

/** What the status card says: a run going on, then a failure, then the games to choose, then
 * the games not on Drive yet, then all synced (or never synced since the sign-in). */
export function statusPill(s: CloudStatus): StatusPill {
  if (s.syncing) return { kind: 'syncing' }
  if (s.error) return { kind: 'error', error: s.error }
  if (s.conflicts.length) return { kind: 'conflicts', n: s.conflicts.length }
  if (s.pending.length) return { kind: 'pending', n: s.pending.length }
  return s.lastSync === null ? { kind: 'never' } : { kind: 'synced' }
}

/** The Drive space bar: shares of the whole space (0–1) and the byte counts its labels show. */
export interface SpaceBar {
  /** EmuDome's own files, as a share of the space. */
  emudome: number
  /** Everything else of the account (Drive, Gmail, Photos), as a share of the space. */
  other: number
  emudomeBytes: number
  otherBytes: number
  /** Bytes still free; null with no limit. */
  free: number | null
  /** The account's space; null = unlimited (some Workspace accounts). */
  limit: number | null
}

const share = (part: number, whole: number) => Math.min(1, Math.max(0, part / whole))

/** The bar from the last measure; null until Drive answered one. */
export function spaceBar(s: CloudStatus): SpaceBar | null {
  if (!s.quota) return null
  const { usage, limit } = s.quota
  const emudomeBytes = Math.min(Math.max(0, s.emudomeBytes ?? 0), usage)
  const otherBytes = Math.max(0, usage - emudomeBytes)
  if (limit === null || limit <= 0)
    return { emudome: 0, other: 0, emudomeBytes, otherBytes, free: null, limit: null }
  const emudome = share(emudomeBytes, limit)
  return {
    emudome,
    // Never past the bar's end, whatever Drive counted.
    other: Math.min(1 - emudome, share(otherBytes, limit)),
    emudomeBytes,
    otherBytes,
    free: Math.max(0, limit - usage),
    limit,
  }
}

/** What a row of «Giochi su Google Drive» offers: the file up, the file down, the file off
 * Drive (its saves stay), or nothing. */
export type GameAction = 'upload' | 'download' | 'remove'

export function gameAction(g: CloudGame): GameAction | null {
  if (g.state === 'localOnly') return 'upload'
  if (g.state === 'driveOnly') return 'download'
  return g.onDrive ? 'remove' : null
}

/** The pill of a game's state, by tone: in both places is the good one, two versions the one to
 * look at. */
export type GameTone = 'ok' | 'warn' | 'quiet'
export function gameTone(state: CloudGameState): GameTone {
  if (state === 'both') return 'ok'
  if (state === 'conflict') return 'warn'
  return 'quiet'
}

/** The games only on Drive, not in this device's library: the library's «Solo su Google Drive». */
export function driveOnly(games: readonly CloudGame[]): CloudGame[] {
  return games.filter(g => g.state === 'driveOnly' && !g.here)
}

/** G05 «Da N minuti»: whole minutes since `since`, at least 1. */
export function minutesSince(since: number, now = Date.now()): number {
  return Math.max(1, Math.round((now - since) / 60_000))
}

/** How far a transfer is, 0–100; null while its size is not known (nothing measured yet). */
export function transferShare(p: CloudProgress | null): number | null {
  if (!p || p.total <= 0) return null
  return Math.min(100, Math.max(0, (p.done / p.total) * 100))
}

/** The progress of a transfer of `game` in `phase`, if that is the one running. */
export function progressOf(
  p: CloudProgress | null,
  phase: CloudProgress['phase'],
  game: GameId | null,
): CloudProgress | null {
  return p && p.phase === phase && (game === null || p.game === game) ? p : null
}

/** A cloud error's identity, for telling it once while it lasts (E07 / E08 / E09). */
export const errorKey = (e: ErrorCode | null): string | null => (e ? e.code : null)
