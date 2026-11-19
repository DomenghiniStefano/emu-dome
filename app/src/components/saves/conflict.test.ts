import { describe, expect, it } from 'vitest'
import { type ConflictVersion, newerIndex } from './conflict'

const version = (savedAt: number): ConflictVersion => ({
  device: 'd',
  savedAt,
  playTimeSecs: null,
  size: 0,
  thumbUrl: null,
})

describe('newerIndex', () => {
  it('picks the side saved last', () => {
    expect(newerIndex([version(1), version(2)])).toBe(1)
    expect(newerIndex([version(2), version(1)])).toBe(0)
  })

  it('picks the first side on a tie', () => {
    expect(newerIndex([version(5), version(5)])).toBe(0)
  })
})
