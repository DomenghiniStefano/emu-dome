// «Gestisci lo spazio» (Trello #29, S1–S3), the pure decisions: the space bar's parts, what each
// game row offers, the quick saves picked. Rust measures and deletes (`storage.rs`); here only
// what the screen shows from its answer.
import type {
  CloudGame,
  GameId,
  StorageBreakdown,
  StorageGame,
  StorageQuickGame,
  StorageQuickPick,
} from '@/emu'

/** The space bar's parts, in its order (S1): EmuDome's three, then the other apps. */
export const BAR_PARTS = ['games', 'quick', 'other', 'apps'] as const
export type BarPart = (typeof BAR_PARTS)[number]

export interface BarSlice {
  part: BarPart
  bytes: number
  /** Its share of the disk, 0–100. */
  percent: number
}

/**
 * The bar's slices over the disk's size; the free room is what is left of the bar. Without the
 * disk's size, the shares are of EmuDome's own bytes (no other apps, no free room).
 */
export function barSlices(b: StorageBreakdown): BarSlice[] {
  const bytes: Record<BarPart, number> = {
    games: b.gamesBytes,
    quick: b.quickBytes,
    other: b.otherBytes,
    apps: b.otherAppsBytes ?? 0,
  }
  const whole = b.total ?? BAR_PARTS.reduce((sum, p) => sum + bytes[p], 0)
  return BAR_PARTS.map(part => ({
    part,
    bytes: bytes[part],
    percent: whole > 0 ? Math.min(100, (bytes[part] / whole) * 100) : 0,
  }))
}

/** What a game row offers (S2, the owner's answer A): EmuDome removes only what it downloaded. */
export type GameAction =
  /** «Togli dal telefono»: in EmuDome's folder and on Drive. */
  | 'remove'
  /** «Apri la cartella»: a user's folder the system can show (the desktop). */
  | 'openFolder'
  /** Nothing: EmuDome's folder but not on Drive (or not known yet), or a picked document. */
  | 'none'

export function gameAction(g: StorageGame, onDrive: boolean): GameAction {
  if (g.location.kind === 'app') return onDrive ? 'remove' : 'none'
  return g.location.openable ? 'openFolder' : 'none'
}

/** The games Drive has, from `cloud_games` (its rows with the game file on Drive). */
export function onDriveSet(games: readonly CloudGame[]): Set<GameId> {
  return new Set(games.filter(g => g.onDrive).map(g => g.id))
}

/** A quick save (its slot) or a screenshot (its name) as a key of a picked set. */
export const pickKey = (game: GameId, item: number | string) => `${game}#${item}`

/** The picked quick saves, in the list's order. */
export function pickedSlots(games: readonly StorageQuickGame[], picked: ReadonlySet<string>) {
  const out: StorageQuickPick[] = []
  for (const g of games)
    for (const s of g.slots)
      if (picked.has(pickKey(g.id, s.slot))) out.push({ game: g.id, slot: s.slot })
  return out
}

/** The bytes of the picked quick saves. */
export function pickedBytes(games: readonly StorageQuickGame[], picked: ReadonlySet<string>) {
  let sum = 0
  for (const g of games)
    for (const s of g.slots) if (picked.has(pickKey(g.id, s.slot))) sum += s.size
  return sum
}

/** A game's quick saves, the heaviest games first (as the screen's games). */
export function byWeight(games: readonly StorageQuickGame[]): StorageQuickGame[] {
  const weight = (g: StorageQuickGame) => g.slots.reduce((sum, s) => sum + s.size, 0)
  return [...games].sort((a, b) => weight(b) - weight(a))
}

/** Every slot of a game picked, or none of them: the game's own box toggles them together. */
export function toggleGame(game: StorageQuickGame, picked: ReadonlySet<string>): Set<string> {
  const next = new Set(picked)
  const keys = game.slots.map(s => pickKey(game.id, s.slot))
  const all = keys.every(k => next.has(k))
  for (const k of keys) {
    if (all) next.delete(k)
    else next.add(k)
  }
  return next
}

/** Whether all, some or none of a game's slots are picked (its box's state). */
export function gamePicked(
  game: StorageQuickGame,
  picked: ReadonlySet<string>,
): boolean | 'indeterminate' {
  const n = game.slots.filter(s => picked.has(pickKey(game.id, s.slot))).length
  if (n === 0) return false
  return n === game.slots.length ? true : 'indeterminate'
}
