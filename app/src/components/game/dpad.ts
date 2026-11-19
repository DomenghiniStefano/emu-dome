import type { Button } from '@/emu'

/** Under this distance from the centre (in half-widths) the d-pad presses nothing. */
export const DPAD_DEADZONE = 0.25

/**
 * The directions a finger at (`x`, `y`) holds on the touch d-pad, both from -1 to 1 across the
 * pad (y downwards). Each direction covers 135°, so the diagonals hold two.
 */
export function dpadDirections(x: number, y: number): Button[] {
  if (Math.hypot(x, y) <= DPAD_DEADZONE) return []
  const angle = Math.atan2(y, x)
  const near = (target: number) =>
    Math.abs(Math.atan2(Math.sin(angle - target), Math.cos(angle - target))) < (Math.PI * 3) / 8
  const held: Button[] = []
  if (near(0)) held.push('right')
  if (near(Math.PI)) held.push('left')
  if (near(Math.PI / 2)) held.push('down')
  if (near(-Math.PI / 2)) held.push('up')
  return held
}
