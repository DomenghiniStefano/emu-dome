import { describe, expect, it } from 'vitest'
import { afterRefusal, effectivePlacement, phoneSend, samePlacement } from './placement'
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

describe('a placement Rust refused', () => {
  const shares = (bottom: number) => ({ ...NONE, bottom })
  const invalid = { code: 'settingInvalidValue', params: { key: 'touch.tscreens' } }

  it('is not sent again: only its inset keeps flowing', () => {
    const sent = { placement: [top], inset: shares(0) }
    expect(phoneSend(sent, { placement: [bottom], inset: shares(0.3) }, [bottom])).toBe('inset')
    expect(phoneSend(sent, { placement: [bottom], inset: shares(0) }, [bottom])).toBeNull()
    expect(
      phoneSend(sent, { placement: [{ ...bottom, x: 0.1 }], inset: shares(0) }, [bottom]),
    ).toBe('pair')
  })

  it('as invalid: Rust holds the old placement and the new inset', () => {
    const before = { placement: [top], inset: shares(0) }
    const tried = { placement: [bottom], inset: shares(0.3) }
    expect(afterRefusal(before, tried, invalid)).toEqual({
      sent: { placement: [top], inset: shares(0.3) },
      refused: [bottom],
    })
  })

  it('for another reason: Rust holds what it held, the same request goes again', () => {
    const before = { placement: [top], inset: shares(0) }
    const tried = { placement: [bottom], inset: shares(0.3) }
    expect(afterRefusal(before, tried, { code: 'noGame', params: {} })).toEqual({
      sent: before,
      refused: null,
    })
  })
})
