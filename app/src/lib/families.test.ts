import { describe, expect, it } from 'vitest'
import type { GameFamily, GameId, LibraryGame } from '@/emu'
import { familyIndex, oneTilePerFamily, versionCount } from './families'

const id = (n: string) => `gba:${n}` as GameId
const game = (n: string) => ({ id: id(n) }) as LibraryGame
const family = (shown: string, ...others: string[]): GameFamily => ({
  id: `gba:${shown.toUpperCase()}`,
  shown: id(shown),
  versions: [shown, ...others].map(id),
  pinned: false,
})

describe('the families index', () => {
  it('tucks every version but the shown one behind its tile', () => {
    const index = familyIndex([family('b', 'a', 'c'), family('e', 'd')])
    const games = ['a', 'b', 'c', 'd', 'e', 'f'].map(game)
    expect(oneTilePerFamily(games, index).map(g => g.id)).toEqual([id('b'), id('e'), id('f')])
    expect(versionCount(index, id('b'))).toBe(3)
    expect(versionCount(index, id('a'))).toBe(3)
    expect(versionCount(index, id('f'))).toBe(0)
  })

  it('leaves the list as it is with no family (grouping off)', () => {
    const games = ['a', 'b'].map(game)
    expect(oneTilePerFamily(games, familyIndex([]))).toBe(games)
  })
})
