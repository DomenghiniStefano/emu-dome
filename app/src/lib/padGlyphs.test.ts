import { describe, expect, it } from 'vitest'
import { actionPos, confirmSide, padGlyph } from './padGlyphs'

describe('actionPos', () => {
  it('confirms on the right face button by default, goes back on the bottom one', () => {
    expect(actionPos('confirm', 'right')).toBe('east')
    expect(actionPos('back', 'right')).toBe('south')
  })

  it('swaps them when the bottom button confirms (pad.confirm)', () => {
    expect(actionPos('confirm', 'bottom')).toBe('south')
    expect(actionPos('back', 'bottom')).toBe('east')
  })

  it('puts the other actions where the routing reads them', () => {
    expect(actionPos('more', 'right')).toBe('north')
    expect(actionPos('search', 'right')).toBe('west')
    expect(actionPos('prev', 'bottom')).toBe('l1')
    expect(actionPos('next', 'bottom')).toBe('r1')
  })

  it('reads pad.confirm, the right side by default', () => {
    expect(confirmSide('bottom')).toBe('bottom')
    expect(confirmSide('right')).toBe('right')
    expect(confirmSide(undefined)).toBe('right')
  })
})

describe('padGlyph', () => {
  it('draws a button as the controller of that family prints it', () => {
    expect(padGlyph('xbox', 'east')).toEqual({ key: 'controls.pos.xbox.east' })
    expect(padGlyph('switch', 'l1')).toEqual({ key: 'controls.pos.switch.l1' })
  })

  it('draws PlayStation faces as shapes, its shoulders by name', () => {
    expect(padGlyph('playstation', 'south')).toEqual({ shape: 'cross' })
    expect(padGlyph('playstation', 'east')).toEqual({ shape: 'circle' })
    expect(padGlyph('playstation', 'west')).toEqual({ shape: 'square' })
    expect(padGlyph('playstation', 'north')).toEqual({ shape: 'triangle' })
    expect(padGlyph('playstation', 'r1')).toEqual({ key: 'controls.pos.common.r1' })
  })

  it('gives a generic controller and none the Nintendo letters of the positions', () => {
    expect(padGlyph('generic', 'east')).toEqual({ key: 'controls.button.a' })
    expect(padGlyph(null, 'south')).toEqual({ key: 'controls.button.b' })
    expect(padGlyph(null, 'north')).toEqual({ key: 'controls.button.x' })
    expect(padGlyph('keyboard', 'west')).toEqual({ key: 'controls.button.y' })
    expect(padGlyph(null, 'l1')).toEqual({ key: 'controls.button.l' })
  })
})
