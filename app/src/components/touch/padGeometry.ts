// The d-pad's drawing (owner, 2026-11-22): one cross-shaped piece with an arrow in each arm, in
// the d-pad box's CSS pixels (spacing.css `dpad`, 116). Pure: TouchDpad.vue draws it, the
// direction under the finger is dpad.ts's (the whole box takes the touch, diagonals included).
import type { Button as GameButton } from '@/emu'

export type Point = readonly [x: number, y: number]

/** A closed path through `points`, each corner rounded by its radius (a quadratic curve from
 * `r` before the corner to `r` after it, on its two edges; convex and concave alike). */
export function roundedPolygon(points: readonly Point[], radii: readonly number[]): string {
  const n = points.length
  const at = (i: number) => points[((i % n) + n) % n]!
  const toward = ([ax, ay]: Point, [bx, by]: Point, d: number): Point => {
    const len = Math.hypot(bx - ax, by - ay)
    return len === 0 ? [ax, ay] : [ax + ((bx - ax) * d) / len, ay + ((by - ay) * d) / len]
  }
  const f = (v: number) => Math.round(v * 100) / 100
  const parts: string[] = []
  for (let i = 0; i < n; i++) {
    const p = at(i)
    const r = radii[i] ?? 0
    const from = toward(p, at(i - 1), r)
    const to = toward(p, at(i + 1), r)
    parts.push(`${i === 0 ? 'M' : 'L'}${f(from[0])} ${f(from[1])}`)
    parts.push(`Q${f(p[0])} ${f(p[1])} ${f(to[0])} ${f(to[1])}`)
  }
  return `${parts.join(' ')} Z`
}

/** The box, an arm's width, the corners' radii and the inset that keeps a 1.5 edge inside. */
export const DPAD_BOX = 116
export const DPAD_ARM = 38
export const DPAD_EDGE = 1.5
const OUTER_R = 6
const INNER_R = 4

const lo = DPAD_EDGE / 2
const hi = DPAD_BOX - DPAD_EDGE / 2
const a = (DPAD_BOX - DPAD_ARM) / 2
const b = (DPAD_BOX + DPAD_ARM) / 2

/** The cross's outline: the arms' ends rounded, the inner corners a little. */
export const DPAD_CROSS = roundedPolygon(
  [
    [a, lo],
    [b, lo],
    [b, a],
    [hi, a],
    [hi, b],
    [b, b],
    [b, hi],
    [a, hi],
    [a, b],
    [lo, b],
    [lo, a],
    [a, a],
  ],
  [
    OUTER_R,
    OUTER_R,
    INNER_R,
    OUTER_R,
    OUTER_R,
    INNER_R,
    OUTER_R,
    OUTER_R,
    INNER_R,
    OUTER_R,
    OUTER_R,
    INNER_R,
  ],
)

type Dir = Extract<GameButton, 'up' | 'down' | 'left' | 'right'>

/** Each arm from its end to the centre (lit while its direction is held; clipped to the cross),
 * and its arrow: a triangle pointing out, near the arm's end. */
export interface DpadArm {
  dir: Dir
  rect: { x: number; y: number; width: number; height: number }
  arrow: string
}

const mid = DPAD_BOX / 2
const reach = mid - DPAD_ARM * 0.42
const t = DPAD_ARM * 0.3

function arrow(dx: number, dy: number): string {
  const [x, y] = [mid + dx * reach, mid + dy * reach]
  const pts: Point[] = dx
    ? [
        [x + dx * t * 0.6, y],
        [x - dx * t * 0.5, y - t * 0.7],
        [x - dx * t * 0.5, y + t * 0.7],
      ]
    : [
        [x, y + dy * t * 0.6],
        [x - t * 0.7, y - dy * t * 0.5],
        [x + t * 0.7, y - dy * t * 0.5],
      ]
  return pts
    .map(([px, py]) => `${Math.round(px * 100) / 100},${Math.round(py * 100) / 100}`)
    .join(' ')
}

export const DPAD_ARMS: readonly DpadArm[] = [
  { dir: 'up', rect: { x: a, y: 0, width: DPAD_ARM, height: mid }, arrow: arrow(0, -1) },
  { dir: 'down', rect: { x: a, y: mid, width: DPAD_ARM, height: mid }, arrow: arrow(0, 1) },
  { dir: 'left', rect: { x: 0, y: a, width: mid, height: DPAD_ARM }, arrow: arrow(-1, 0) },
  { dir: 'right', rect: { x: mid, y: a, width: mid, height: DPAD_ARM }, arrow: arrow(1, 0) },
]
