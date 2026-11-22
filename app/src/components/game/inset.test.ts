import { describe, expect, it } from 'vitest'
import { bottomInset, insetShares, NO_INSET, sameInset } from './inset'

describe('insetShares', () => {
  it('is each edge as a share of its side of the window, 0.9 at most', () => {
    expect(insetShares({ ...NO_INSET, left: 186, right: 182 }, 892, 412)).toEqual({
      left: 186 / 892,
      top: 0,
      right: 182 / 892,
      bottom: 0,
      align: 'centre',
      between: 0,
    })
    expect(insetShares({ ...bottomInset(330), align: 'bottom' }, 412, 892)).toEqual({
      ...insetShares(NO_INSET, 1, 1),
      bottom: 330 / 892,
      align: 'bottom',
    })
    expect(insetShares(bottomInset(5000), 800, 600).bottom).toBe(0.9)
    expect(insetShares({ ...NO_INSET, top: -20, left: Number.NaN }, 800, 600)).toEqual(
      insetShares(NO_INSET, 800, 600),
    )
  })

  it('takes the room between two screens as a share of the height, half at most', () => {
    expect(insetShares({ ...NO_INSET, between: 44 }, 412, 892).between).toBe(44 / 892)
    expect(insetShares({ ...NO_INSET, between: 8 }, 892, 412).between).toBe(8 / 412)
    expect(insetShares({ ...NO_INSET, between: 900 }, 412, 892).between).toBe(0.5)
  })

  it('keeps nothing of an empty window', () => {
    expect(insetShares(bottomInset(100), 0, 0)).toEqual(insetShares(NO_INSET, 1, 1))
  })
})

describe('sameInset', () => {
  it('compares every edge and the way the game sits', () => {
    expect(sameInset(bottomInset(10), bottomInset(10))).toBe(true)
    expect(sameInset(bottomInset(10), { ...bottomInset(10), align: 'bottom' })).toBe(false)
    expect(sameInset(NO_INSET, { ...NO_INSET, right: 1 })).toBe(false)
    expect(sameInset(NO_INSET, { ...NO_INSET, between: 44 })).toBe(false)
  })
})
