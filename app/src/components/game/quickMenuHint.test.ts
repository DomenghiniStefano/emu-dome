import { describe, expect, it } from 'vitest'
import { quickMenuHintKeys } from './quickMenuHint'

describe('quickMenuHintKeys', () => {
  it('names Esc and Home on a computer, whatever is connected', () => {
    for (const keyboard of [false, true])
      for (const controller of [false, true])
        expect(quickMenuHintKeys(false, keyboard, controller)).toEqual({ esc: true, home: true })
  })
  it('does not show on a phone with neither a keyboard nor a controller', () => {
    expect(quickMenuHintKeys(true, false, false)).toBeNull()
  })
  it('names on a phone only the input that is there', () => {
    expect(quickMenuHintKeys(true, false, true)).toEqual({ esc: false, home: true })
    expect(quickMenuHintKeys(true, true, false)).toEqual({ esc: true, home: false })
    expect(quickMenuHintKeys(true, true, true)).toEqual({ esc: true, home: true })
  })
})
