import { describe, expect, it } from 'vitest'
import { NO_INSET } from '@/components/game/inset'
import { MenuInset, panelInset } from './panel'

describe('panelInset', () => {
  const upright = { width: 412, height: 892 }
  const sideways = { width: 892, height: 412 }

  it('upright: the game against the top of the sheet', () => {
    expect(panelInset(MenuInset.Sheet, { left: 0, top: 318.7 }, upright)).toEqual({
      ...NO_INSET,
      bottom: 574,
      align: 'bottom',
    })
  })

  it('on the side: the game centred left of the panel', () => {
    expect(panelInset(MenuInset.Side, { left: 452, top: 0 }, sideways)).toEqual({
      ...NO_INSET,
      right: 440,
    })
  })

  it('keeps nothing for a panel off the window or with none', () => {
    expect(panelInset(MenuInset.Side, { left: 1000, top: 0 }, sideways)).toEqual(NO_INSET)
    expect(panelInset(MenuInset.None, { left: 10, top: 10 }, sideways)).toEqual(NO_INSET)
  })
})
