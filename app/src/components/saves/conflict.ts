// V06: one side of a save conflict, and which side is the newer one.

/** One side of the conflict. */
export interface ConflictVersion {
  /** «Questo computer», «Galaxy A71»: the device's own name, from the sync step. */
  device: string
  /** Unix ms. */
  savedAt: number
  playTimeSecs: number | null
  size: number
  /** The last screen, when the version has a quick save with a thumbnail. */
  thumbUrl: string | null
}

/** The side saved last; the first one on a tie. */
export function newerIndex(versions: readonly [ConflictVersion, ConflictVersion]): 0 | 1 {
  return versions[1].savedAt > versions[0].savedAt ? 1 : 0
}
