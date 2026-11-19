import { describe, expect, it } from 'vitest'
import { timeAgo } from './timeAgo'

const NOW = 1_795_000_000_000
const MINUTE = 60_000

describe('timeAgo', () => {
  it('counts minutes under an hour', () => {
    expect(timeAgo(NOW - 2 * MINUTE, NOW, 'en')).toBe('2 minutes ago')
    expect(timeAgo(NOW - 59 * MINUTE, NOW, 'en')).toBe('59 minutes ago')
  })
  it('says this minute under half a minute', () => {
    expect(timeAgo(NOW - 10_000, NOW, 'en')).toBe('this minute')
  })
  it('counts hours from the hour on', () => {
    expect(timeAgo(NOW - 60 * MINUTE, NOW, 'en')).toBe('1 hour ago')
    expect(timeAgo(NOW - 180 * MINUTE, NOW, 'en')).toBe('3 hours ago')
  })
  it('speaks the locale', () => {
    expect(timeAgo(NOW - 2 * MINUTE, NOW, 'it')).toBe('2 minuti fa')
  })
})
