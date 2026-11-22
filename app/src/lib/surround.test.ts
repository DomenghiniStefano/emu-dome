import { describe, expect, it } from 'vitest'
import { nativeGameRect } from './surround'

const none = { left: 0, top: 0, right: 0, bottom: 0, align: 'centre' as const, between: 0 }
const gba = { w: 240, h: 160 }

// The cases of `screens::fit`'s tests (crates/emudome-core/src/screens.rs): the mirror must agree.
describe('nativeGameRect', () => {
  it('fits at an integer scale, centred in the whole surface', () => {
    expect(nativeGameRect({ w: 1440, h: 900 }, gba, none)).toEqual({
      x: 120,
      y: 50,
      w: 1200,
      h: 800,
    })
  })

  it('keeps the bottom share clear', () => {
    const r = nativeGameRect({ w: 1440, h: 900 }, gba, { ...none, bottom: 0.3 })
    expect(r.y + r.h).toBeLessThanOrEqual(630)
  })

  it('sits against the bottom edge when asked, with no band above the controls', () => {
    const r = nativeGameRect({ w: 412, h: 892 }, gba, {
      ...none,
      bottom: 368 / 892,
      align: 'bottom',
    })
    // f64 here, f32 in Rust: within a pixel of the controls' band.
    expect(Math.abs(r.y + r.h - 524)).toBeLessThanOrEqual(1)
    expect(r.x).toBe(Math.floor((412 - r.w) / 2))
  })

  it('is centred between side columns, not in the surface', () => {
    const [w, h] = [2341.5, 1081.5]
    const r = nativeGameRect({ w, h }, gba, { ...none, left: 211 / 892, right: 204 / 892 })
    expect([r.w, r.h]).toEqual([1200, 800])
    const [left, right] = [(w * 211) / 892, w - (w * 204) / 892]
    expect(Math.abs(r.x + r.w / 2 - (left + right) / 2)).toBeLessThanOrEqual(1)
    expect(r.y).toBe(Math.floor((h - 800) / 2))
  })

  it('always leaves a tenth of each axis', () => {
    const r = nativeGameRect(
      { w: 1000, h: 1000 },
      { w: 100, h: 100 },
      {
        left: 0.7,
        right: 0.7,
        top: Number.NaN,
        bottom: 2,
        align: 'centre',
        between: 0,
      },
    )
    expect([r.x, r.y]).toEqual([450, 0])
    expect(r.w).toBeCloseTo(100)
    expect(r.h).toBeCloseTo(100)
  })
})
