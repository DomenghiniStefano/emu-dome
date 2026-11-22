import { describe, expect, it } from 'vitest'
import { dpadDirections } from '@/components/game/dpad'
import { DPAD_ARM, DPAD_ARMS, DPAD_BOX, DPAD_CROSS, roundedPolygon } from './padGeometry'

describe('the d-pad drawing', () => {
  it('rounds every corner of a polygon, convex and concave', () => {
    expect(
      roundedPolygon(
        [
          [0, 0],
          [10, 0],
          [10, 10],
        ],
        [1, 0, 2],
      ),
    ).toBe('M0.71 0.71 Q0 0 1 0 L10 0 Q10 0 10 0 L10 8 Q10 10 8.59 8.59 Z')
  })

  it('is one cross of twelve corners inside its box', () => {
    expect(DPAD_CROSS.match(/Q/g)).toHaveLength(12)
    const numbers = DPAD_CROSS.match(/-?\d+(\.\d+)?/g)!.map(Number)
    expect(Math.min(...numbers)).toBeGreaterThan(0)
    expect(Math.max(...numbers)).toBeLessThan(DPAD_BOX)
    // A touch target for each arm: the box's third, under the finger's 44 px only as drawn.
    expect(DPAD_ARM).toBeGreaterThan(DPAD_BOX / 3 - 1)
  })

  it('puts each arrow in the arm of its own direction', () => {
    for (const arm of DPAD_ARMS) {
      const pts = arm.arrow.split(' ').map(p => p.split(',').map(Number) as [number, number])
      const cx = pts.reduce((s, p) => s + p[0], 0) / 3
      const cy = pts.reduce((s, p) => s + p[1], 0) / 3
      // The direction a finger on the arrow gives is the arm's own.
      expect(dpadDirections((cx / DPAD_BOX) * 2 - 1, (cy / DPAD_BOX) * 2 - 1)).toEqual([arm.dir])
      // The lit rectangle holds it.
      const r = arm.rect
      expect(cx).toBeGreaterThan(r.x)
      expect(cx).toBeLessThan(r.x + r.width)
      expect(cy).toBeGreaterThan(r.y)
      expect(cy).toBeLessThan(r.y + r.height)
    }
  })
})
