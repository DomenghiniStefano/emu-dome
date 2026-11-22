import { describe, expect, it } from 'vitest'
import type { GameFamily, GameId, LibraryGame } from '@/emu'
import { regionTagKey, versionRows } from './gameVersions'

const id = (n: string) => `nds:${n}` as GameId
const game = (n: string) => ({ id: id(n), title: n }) as LibraryGame
const family: GameFamily = {
  id: 'nds:ADA',
  shown: id('us'),
  versions: [id('us'), id('eu'), id('jp')],
  pinned: false,
}

describe('versionRows', () => {
  it('lists the versions in the family’s order, the tile’s one marked', () => {
    const rows = versionRows(family, [game('jp'), game('eu'), game('us'), game('other')])
    expect(rows.map(r => [r.game.id, r.inLibrary])).toEqual([
      [id('us'), true],
      [id('eu'), false],
      [id('jp'), false],
    ])
  })

  it('leaves out a version the library no longer lists, and is empty with no family', () => {
    expect(versionRows(family, [game('us'), game('jp')]).map(r => r.game.id)).toEqual([
      id('us'),
      id('jp'),
    ])
    expect(versionRows(undefined, [game('us')])).toEqual([])
  })
})

describe('regionTagKey', () => {
  it('names the region’s tag, or none', () => {
    expect(regionTagKey('europe')).toBe('details.versions.regionTag.europe')
    expect(regionTagKey(null)).toBe('details.versions.regionTag.none')
  })
})
