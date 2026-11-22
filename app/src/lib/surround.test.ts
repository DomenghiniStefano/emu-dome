import { describe, expect, it } from 'vitest'
import { holesClipPath } from './surround'

const OUTER = 'polygon(evenodd, 0 0, 100% 0, 100% 100%, 0 100%, 0 0, '

/** The holes' corners as the path lists them: [x1, y1, x2, y2] per hole. */
function holesOf(path: string | undefined): number[][] {
  expect(path?.startsWith(OUTER)).toBe(true)
  const points = (path ?? '')
    .slice(OUTER.length, -1)
    .split(', ')
    .map(p => p.split(' ').map(v => parseFloat(v)))
  const holes: number[][] = []
  // Each hole: five corners (closed), then back to the window's corner.
  expect(points.length % 6).toBe(0)
  for (let i = 0; i < points.length; i += 6) {
    const [a, , c] = [points[i], points[i + 1], points[i + 2]]
    expect(points[i + 5]).toEqual([0, 0])
    holes.push([a?.[0] ?? NaN, a?.[1] ?? NaN, c?.[0] ?? NaN, c?.[1] ?? NaN])
  }
  return holes
}

// The holes come from Rust's `screens_placements` (one per screen): no geometry is worked out here.
describe('holesClipPath', () => {
  it('cuts nothing without a screen', () => {
    expect(holesClipPath([], 1)).toBeUndefined()
    expect(holesClipPath([{ x: 10, y: 10, w: 0, h: 50 }], 1)).toBeUndefined()
  })

  it('cuts one hole for one screen', () => {
    expect(holesOf(holesClipPath([{ x: 120, y: 50, w: 1200, h: 800 }], 1))).toEqual([
      [120, 50, 1320, 850],
    ])
  })

  it('cuts one hole per screen, the room between them painted (two screens apart)', () => {
    const top = { x: 0, y: 32, w: 412, h: 309 }
    const bottom = { x: 0, y: 385, w: 412, h: 309 }
    expect(holesOf(holesClipPath([top, bottom], 1))).toEqual([
      [0, 32, 412, 341],
      [0, 385, 412, 694],
    ])
  })

  it('rounds each hole outwards to whole device pixels', () => {
    // 2.625 device px per CSS px: 100.2 → 263.025 device px, floored to 263 → 100.190…
    const [hole] = holesOf(holesClipPath([{ x: 100.2, y: 10, w: 50.1, h: 20 }], 2.625))
    expect(hole?.[0]).toBeCloseTo(263 / 2.625)
    expect(hole?.[2]).toBeCloseTo(Math.ceil(150.3 * 2.625) / 2.625)
    expect(hole?.[0] ?? 0).toBeLessThanOrEqual(100.2)
    expect(hole?.[2] ?? 0).toBeGreaterThanOrEqual(150.3)
  })

  it('merges screens that overlap into one hole (evenodd would paint their overlap)', () => {
    const big = { x: 0, y: 0, w: 400, h: 300 }
    const pip = { x: 300, y: 200, w: 200, h: 150 }
    const apart = { x: 600, y: 0, w: 100, h: 100 }
    expect(holesOf(holesClipPath([big, apart, pip], 1))).toEqual([
      [0, 0, 500, 350],
      [600, 0, 700, 100],
    ])
  })
})
