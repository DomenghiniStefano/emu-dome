/** The done part of a progress bar as a CSS width, clamped to the track; null while the total is
 * not known (indeterminate). */
export function fillWidth(value: number | null | undefined, max: number): string | null {
  if (value === null || value === undefined || !(max > 0)) return null
  return `${Math.min(100, Math.max(0, (value / max) * 100))}%`
}
