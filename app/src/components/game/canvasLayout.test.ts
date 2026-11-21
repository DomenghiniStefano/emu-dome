import { describe, expect, it } from 'vitest'
import { canvasSize } from './canvasLayout'

const GBA = { w: 240, h: 160 }
const NDS = { w: 256, h: 384 }

describe('canvasSize', () => {
  it('scales by whole steps when the game fits at least once', () => {
    // 1920×1080: 8× by width, 6.75× by height → 6×.
    expect(canvasSize(GBA, { w: 1920, h: 1080 }, 0, 1)).toEqual({ w: 1440, h: 960 })
  })

  it('shrinks to fit, not by steps, when the window is smaller than the game', () => {
    expect(canvasSize(NDS, { w: 800, h: 192 }, 0, 1)).toEqual({ w: 128, h: 192 })
  })

  it('keeps the quick menu bar clear at the bottom', () => {
    // 1080 - 200 = 880 → 5× by height.
    expect(canvasSize(GBA, { w: 1920, h: 1080 }, 200, 1)).toEqual({ w: 1200, h: 800 })
    // A bar taller than the window leaves 1 px, never a negative size.
    expect(canvasSize(GBA, { w: 1920, h: 100 }, 400, 1)?.h).toBeCloseTo(1)
  })

  it('writes CSS px that the interface zoom turns back into the same screen px', () => {
    const size = canvasSize(GBA, { w: 1920, h: 1080 }, 0, 1.25)
    expect(size).toEqual({ w: 1440 / 1.25, h: 960 / 1.25 })
  })

  it('has no size before the first frame', () => {
    expect(canvasSize({ w: 0, h: 0 }, { w: 1920, h: 1080 }, 0, 1)).toBeNull()
  })
})
