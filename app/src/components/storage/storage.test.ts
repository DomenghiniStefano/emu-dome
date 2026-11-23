import { describe, expect, it } from 'vitest'
import type { CloudGame, StorageBreakdown, StorageGame, StorageQuickGame } from '@/emu'
import {
  barSlices,
  byWeight,
  gameAction,
  gamePicked,
  onDriveSet,
  pickKey,
  pickedBytes,
  pickedSlots,
  toggleGame,
} from './storage'

const GIB = 1024 ** 3

function breakdown(over: Partial<StorageBreakdown> = {}): StorageBreakdown {
  return {
    free: 10 * GIB,
    total: 100 * GIB,
    gamesBytes: 20 * GIB,
    quickBytes: 5 * GIB,
    otherBytes: 5 * GIB,
    otherAppsBytes: 60 * GIB,
    games: [],
    quickSlots: 0,
    quickGames: 0,
    screenshotBytes: 0,
    screenshotCount: 0,
    screenshotGames: [],
    tempBytes: 0,
    ...over,
  }
}

const game = (location: StorageGame['location']): StorageGame => ({
  id: 'gba:1a2b',
  title: 'Game',
  console: 'gba',
  size: 1,
  location,
})

describe('the space bar', () => {
  it('splits the disk into its parts, the free room left over', () => {
    const s = barSlices(breakdown())
    expect(s.map(x => [x.part, Math.round(x.percent)])).toEqual([
      ['games', 20],
      ['quick', 5],
      ['other', 5],
      ['apps', 60],
    ])
  })

  it('without the disk size shares only what EmuDome has', () => {
    const s = barSlices(breakdown({ total: null, free: null, otherAppsBytes: null }))
    expect(s.map(x => Math.round(x.percent))).toEqual([67, 17, 17, 0])
  })
})

describe('what a game row offers', () => {
  it('removes only a game EmuDome downloaded that Drive has too', () => {
    expect(gameAction(game({ kind: 'app' }), true)).toBe('remove')
    expect(gameAction(game({ kind: 'app' }), false)).toBe('none')
  })

  it('never removes a file of the user, it opens its folder where it can', () => {
    expect(gameAction(game({ kind: 'folder', name: 'Roms', openable: true }), true)).toBe(
      'openFolder',
    )
    expect(gameAction(game({ kind: 'folder', name: 'Download', openable: false }), true)).toBe(
      'none',
    )
  })

  it('reads Drive from the games whose file is there', () => {
    const row = (id: string, onDrive: boolean) => ({ id, onDrive }) as CloudGame
    expect([...onDriveSet([row('gba:1', true), row('gba:2', false)])]).toEqual(['gba:1'])
  })
})

describe('the quick saves picked', () => {
  const gba: StorageQuickGame = {
    id: 'gba:1',
    slots: [
      { slot: 0, size: 10, savedAt: null },
      { slot: 1, size: 20, savedAt: 5 },
    ],
  }
  const nds: StorageQuickGame = { id: 'nds:2', slots: [{ slot: 3, size: 100, savedAt: null }] }
  const games = [gba, nds]

  it('lists the heaviest games first', () => {
    expect(byWeight(games).map(g => g.id)).toEqual(['nds:2', 'gba:1'])
  })

  it('picks a whole game with its box, and unpicks it', () => {
    let picked = toggleGame(gba, new Set())
    expect(gamePicked(gba, picked)).toBe(true)
    expect(pickedSlots(games, picked)).toEqual([
      { game: 'gba:1', slot: 0 },
      { game: 'gba:1', slot: 1 },
    ])
    expect(pickedBytes(games, picked)).toBe(30)
    picked = toggleGame(gba, picked)
    expect(picked.size).toBe(0)
  })

  it('shows a game partly picked, and its box then picks the rest', () => {
    const one = new Set([pickKey('gba:1', 1)])
    expect(gamePicked(gba, one)).toBe('indeterminate')
    expect(gamePicked(nds, one)).toBe(false)
    expect(toggleGame(gba, one).size).toBe(2)
  })
})
