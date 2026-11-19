// L1 / R1 through a list that wraps around at its ends: tabs (game details, Controls) and levels
// (Settings).

/** The item `dir` steps away from `current`, round the ends; `current` when it is not listed. */
export function cycle<T>(list: readonly T[], current: T, dir: -1 | 1): T {
  const at = list.indexOf(current)
  if (at < 0) return current
  return list[(at + dir + list.length) % list.length] as T
}
