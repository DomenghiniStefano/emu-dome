// The touch Circle Pad's value (Trello #29), pure: a finger at (x, y), each -1..1 across the
// ring (y downwards), held to the unit circle, with a small dead zone in the middle so a resting
// thumb does not drift.

/** Under this distance from the centre the stick stays centred. */
export const STICK_DEADZONE = 0.12

/** The stick's value for a finger at (`x`, `y`): inside the circle, 0 in the dead zone, values
 * rounded to 3 decimals. */
export function stickAt(x: number, y: number): [number, number] {
  const r = Math.hypot(x, y)
  if (!Number.isFinite(r) || r <= STICK_DEADZONE) return [0, 0]
  const k = r > 1 ? 1 / r : 1
  const round = (v: number) => Math.round(v * 1000) / 1000 || 0
  return [round(x * k), round(y * k)]
}
