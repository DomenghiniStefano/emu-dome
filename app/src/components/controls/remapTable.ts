// C02's table: how many rows use each source, and what a row's status column says.
import type { RemapRow } from '../../emu'

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
