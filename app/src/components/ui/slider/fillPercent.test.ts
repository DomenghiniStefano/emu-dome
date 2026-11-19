import { describe, expect, it } from 'vitest'
import { fillPercent } from './variants'

describe('fillPercent', () => {
  it('places the value along the range', () => {
    expect(fillPercent(40, 0, 80)).toBe(50)
    expect(fillPercent(15, 10, 90)).toBe(6.25)
  })

  it('stays inside the track', () => {
    expect(fillPercent(-5, 0, 100)).toBe(0)
    expect(fillPercent(120, 0, 100)).toBe(100)
  })

  it('is empty on an empty range', () => {
    expect(fillPercent(3, 5, 5)).toBe(0)
  })
})
