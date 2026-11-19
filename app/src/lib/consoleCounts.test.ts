import { describe, expect, it } from 'vitest'
import { consoleCounts, countByConsole } from './consoleCounts'

describe('countByConsole', () => {
  it('counts the games of each console', () => {
    expect(
      countByConsole([
        { console: 'gba' },
        { console: 'gb' },
        { console: 'gba' },
        { console: 'nds' },
      ]),
    ).toEqual({ gba: 2, gb: 1, nds: 1 })
  })
})

describe('consoleCounts', () => {
  it('has one tile per console, Game Boy and Color together, in order', () => {
    const tiles = consoleCounts({ gba: 2, gb: 1, gbc: 1, nds: 1 })
    expect(tiles.map(t => [t.id, t.count])).toEqual([
      ['gba', 2],
      ['nds', 1],
      ['3ds', 0],
      ['gb', 2],
    ])
    expect(tiles[3]?.label).toBe('library.added.gbAndColor')
  })

  it('leaves out the empty tiles when asked', () => {
    expect(consoleCounts({ nds: 1, gb: 1, gbc: 2 }, { hideEmpty: true })).toEqual([
      { id: 'nds', label: 'console.nds', count: 1 },
      { id: 'gb', label: 'library.added.gbAndColor', count: 3 },
    ])
    expect(consoleCounts({}, { hideEmpty: true })).toEqual([])
  })
})
