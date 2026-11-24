import { describe, expect, it } from 'vitest'
import { effectivePlacement, samePlacement } from './placement'

const top = { id: 'top', x: 0, y: 0, w: 1, h: 0.4 } as const
const bottom = { id: 'bottom', x: 0, y: 0.5, w: 1, h: 0.4 } as const

describe('placement', () => {
  it("the editor's placement wins while open and the layer's comes back", () => {
    expect(effectivePlacement([top, bottom], [bottom])).toEqual([bottom])
    expect(effectivePlacement([top, bottom], [])).toEqual([])
    expect(effectivePlacement([top, bottom], null)).toEqual([top, bottom])
  })
  it('tells the same placement apart from a moved one', () => {
    expect(samePlacement([top, bottom], [{ ...top }, { ...bottom }])).toBe(true)
    expect(samePlacement([top, bottom], [{ ...top, x: 0.1 }, bottom])).toBe(false)
    expect(samePlacement([top], [])).toBe(false)
  })
})
