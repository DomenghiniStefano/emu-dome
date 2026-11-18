import { describe, expect, it } from 'vitest'
import { consoleTiles, readyCovers } from './library'

describe('consoleTiles', () => {
  it('counts each console, Game Boy and Color together', () => {
    const tiles = consoleTiles([
      { console: 'gba' },
      { console: 'gb' },
      { console: 'gbc' },
      { console: 'nds' },
      { console: 'gba' },
    ])
    expect(tiles.map(t => [t.id, t.count])).toEqual([
      ['gba', 2],
      ['nds', 1],
      ['3ds', 0],
      ['gb', 2],
    ])
    expect(tiles[3]?.label).toBe('library.added.gbAndColor')
  })
})

describe('readyCovers', () => {
  it('takes three visible games by title', () => {
    const games = [
      { title: 'Delta', hidden: false },
      { title: 'Alpha', hidden: true },
      { title: 'Charlie', hidden: false },
      { title: 'Bravo', hidden: false },
      { title: 'Echo', hidden: false },
    ]
    expect(readyCovers(games).map(g => g.title)).toEqual(['Bravo', 'Charlie', 'Delta'])
  })
})
