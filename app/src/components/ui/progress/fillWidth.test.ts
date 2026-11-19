import { describe, expect, it } from 'vitest'
import { fillWidth } from './fillWidth'

describe('fillWidth', () => {
  it('is null while the total is not known', () => {
    expect(fillWidth(null, 100)).toBeNull()
    expect(fillWidth(undefined, 100)).toBeNull()
  })
  it('is the share of the track, clamped', () => {
    expect(fillWidth(25, 100)).toBe('25%')
    expect(fillWidth(3, 4)).toBe('75%')
    expect(fillWidth(140, 100)).toBe('100%')
    expect(fillWidth(-5, 100)).toBe('0%')
  })
  it('is null for a max that is not a positive number', () => {
    expect(fillWidth(1, 0)).toBeNull()
  })
})
