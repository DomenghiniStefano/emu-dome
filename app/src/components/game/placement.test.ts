import { describe, expect, it } from 'vitest'
import { effectivePlacement, phoneSend, samePlacement } from './placement'
import { insetShares, NO_INSET } from './inset'

const NONE = insetShares(NO_INSET, 1, 1)

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

describe('what Rust is sent of the phone view', () => {
  const shares = (bottom: number) => ({ ...NONE, bottom })
  it('the placement and its inset together when the placement changed', () => {
    const sent = { placement: [top], inset: shares(0) }
    expect(phoneSend(sent, { placement: [bottom], inset: shares(0.3) })).toBe('pair')
    expect(phoneSend(sent, { placement: [bottom], inset: shares(0) })).toBe('pair')
  })
  it('the inset alone when only it changed, nothing when neither did', () => {
    const sent = { placement: [top], inset: shares(0) }
    expect(phoneSend(sent, { placement: [{ ...top }], inset: shares(0.3) })).toBe('inset')
    expect(phoneSend(sent, { placement: [{ ...top }], inset: shares(0) })).toBeNull()
  })
})
