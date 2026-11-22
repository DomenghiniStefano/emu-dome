import { describe, expect, it } from 'vitest'
import { stickAt } from './stick'

describe('the touch Circle Pad', () => {
  it('stays centred in the dead zone', () => {
    expect(stickAt(0.05, -0.05)).toEqual([0, 0])
    expect(stickAt(Number.NaN, 0)).toEqual([0, 0])
  })

  it('follows the finger inside the ring and stops at its edge', () => {
    expect(stickAt(0.5, 0)).toEqual([0.5, 0])
    expect(stickAt(3, 4)).toEqual([0.6, 0.8])
    expect(stickAt(-2, 0)).toEqual([-1, 0])
  })
})
