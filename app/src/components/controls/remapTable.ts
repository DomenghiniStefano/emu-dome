// C02's table: how many rows use each source, and what a row's status column says.
import type { RemapRow } from '@/emu'

/** How many rows use each source: a source used twice is flagged. */
export function sourceUses(rows: RemapRow[]): Map<string, number> {
  const uses = new Map<string, number>()
  for (const r of rows) if (r.source) uses.set(r.source, (uses.get(r.source) ?? 0) + 1)
  return uses
}

export const RemapRowStatus = {
  /** Nothing to say, or the row is listening (its field says it). */
  None: 'none',
  /** «Usato anche da un altro»: in the warning colour. */
  UsedTwice: 'usedTwice',
  /** «Modificato». */
  Changed: 'changed',
} as const
export type RemapRowStatus = (typeof RemapRowStatus)[keyof typeof RemapRowStatus]

export function remapRowStatus(
  row: RemapRow,
  listening: boolean,
  uses: Map<string, number>,
): RemapRowStatus {
  if (listening) return RemapRowStatus.None
  if (isUsedTwice(row, uses)) return RemapRowStatus.UsedTwice
  return row.changed ? RemapRowStatus.Changed : RemapRowStatus.None
}

export const isUsedTwice = (row: RemapRow, uses: Map<string, number>): boolean =>
  !!row.source && (uses.get(row.source) ?? 0) > 1

// ---- Capturing a source for a row (C02's buttons, C03's shortcuts) ----
//
// Both tables bind an id (a game button, a shortcut) to a source (a position or a key). A source
// already given to another row is not taken from it silently: the screen asks to swap the two.

/** One row of such a table: its id and the source it has (null: none). */
export interface Binding<K extends string, S extends string> {
  id: K
  source: S | null
}

/** The changes one write makes: id → its new source (null: none). */
export type BindingChanges<K extends string, S extends string> = Partial<Record<K, S | null>>

/** «… è già … Scambiali?»: `source`, captured for `id`, is the source of `other`. */
export interface BindingConflict<K extends string, S extends string> {
  id: K
  other: K
  source: S
}

/** What capturing `source` for `id` does. */
export type CaptureOutcome<K extends string, S extends string> =
  | { kind: 'unchanged' }
  | { kind: 'conflict'; conflict: BindingConflict<K, S> }
  | { kind: 'assign'; changes: BindingChanges<K, S> }

/** Nothing (no such row, or the source it already has), a conflict, or one change. */
export function captureOutcome<K extends string, S extends string>(
  rows: readonly Binding<K, S>[],
  id: K,
  source: S,
): CaptureOutcome<K, S> {
  const row = rows.find(r => r.id === id)
  if (!row || row.source === source) return { kind: 'unchanged' }
  const other = rows.find(r => r.id !== id && r.source === source)
  if (other) return { kind: 'conflict', conflict: { id, other: other.id, source } }
  return { kind: 'assign', changes: { [id]: source } as BindingChanges<K, S> }
}

/** «Scambiali»: the captured source to its row, that row's old source to the other one. */
export function swapChanges<K extends string, S extends string>(
  rows: readonly Binding<K, S>[],
  c: BindingConflict<K, S>,
): BindingChanges<K, S> {
  const mine = rows.find(r => r.id === c.id)?.source ?? null
  return { [c.id]: c.source, [c.other]: mine } as BindingChanges<K, S>
}
