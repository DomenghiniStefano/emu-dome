import { describe, expect, it, vi } from 'vitest'
import { AUTO_SLOT, type LibraryFolder, type LibraryGame } from '@/emu'
import catalog from '@/generated/settings-catalog.json'
import {
  GAMES_VIEWS,
  consoleChips,
  coverKinds,
  isGamesView,
  ofConsole,
  recentlyPlayed,
  resumeSource,
  resumeTarget,
  unreachableFolders,
} from './libraryView'

// emu.ts reaches for the Tauri bridge when it loads; these functions need only its constant.
vi.mock('@/emu', () => ({ AUTO_SLOT: 0 }))

const game = (id: string, over: Partial<LibraryGame> = {}): LibraryGame =>
  ({
    id,
    console: 'gba',
    title: id,
    folder: 'C:/Games',
    available: true,
    hidden: false,
    lastPlayed: null,
    hasIcon: false,
    ...over,
  }) as LibraryGame

describe('consoleChips', () => {
  it('puts «Tutti» first, then only the consoles present, in console order', () => {
    const games = [
      game('a', { console: 'nds' }),
      game('b', { console: 'gb' }),
      game('c', { console: 'nds' }),
    ]
    expect(consoleChips(games)).toEqual([
      { id: 'all', count: 3 },
      { id: 'gb', count: 1 },
      { id: 'nds', count: 2 },
    ])
  })

  it('has only «Tutti» with no games', () => {
    expect(consoleChips([])).toEqual([{ id: 'all', count: 0 }])
  })
})

describe('ofConsole', () => {
  const games = [game('a', { console: 'gb' }), game('b', { console: 'gba' })]
  it('keeps every game for «Tutti»', () => {
    expect(ofConsole(games, 'all')).toBe(games)
  })
  it('keeps one console', () => {
    expect(ofConsole(games, 'gba').map(g => g.id)).toEqual(['b'])
  })
})

describe('recentlyPlayed', () => {
  it('takes the games played last, newest first, leaving out hidden and never played', () => {
    const games = [
      game('old', { lastPlayed: 1 }),
      game('never'),
      game('hidden', { lastPlayed: 9, hidden: true }),
      game('new', { lastPlayed: 5 }),
    ]
    expect(recentlyPlayed(games).map(g => g.id)).toEqual(['new', 'old'])
  })

  it('stops at the count', () => {
    const games = [1, 2, 3, 4, 5].map(n => game(`g${n}`, { lastPlayed: n }))
    expect(recentlyPlayed(games).map(g => g.id)).toEqual(['g5', 'g4', 'g3', 'g2'])
    expect(recentlyPlayed(games, 1).map(g => g.id)).toEqual(['g5'])
  })
})

describe('unreachableFolders', () => {
  it('lists the folders that are gone with how many games they hold', () => {
    const folders: LibraryFolder[] = [
      { path: 'C:/Games', name: 'C:/Games', reachable: true },
      { path: 'D:/Old', name: 'D:/Old', reachable: false },
      { path: 'content://x/tree/primary%3AROMs', name: 'ROMs', reachable: false },
    ]
    const games = [game('a', { folder: 'D:/Old' }), game('b'), game('c', { folder: 'D:/Old' })]
    expect(unreachableFolders(folders, games)).toEqual([
      { path: 'D:/Old', name: 'D:/Old', count: 2 },
      // A picked Android folder: shown by its name, its games found by its URI.
      { path: 'content://x/tree/primary%3AROMs', name: 'ROMs', count: 0 },
    ])
  })
})

describe('resumeSource', () => {
  it('says nothing when there is no save, or no point yet', () => {
    expect(resumeSource(undefined)).toBeNull()
    expect(resumeSource({ slot: null, thumbUrl: null, none: true })).toBeNull()
  })

  it('names the in-game save, the automatic save or the slot', () => {
    expect(resumeSource({ slot: null, thumbUrl: null, none: false })).toEqual({
      key: 'library.continue.fromInGame',
    })
    expect(resumeSource({ slot: AUTO_SLOT, thumbUrl: null, none: false })).toEqual({
      key: 'library.continue.fromAuto',
    })
    expect(resumeSource({ slot: 3, thumbUrl: null, none: false })).toEqual({
      key: 'library.continue.fromSlot',
      params: { slot: 3 },
    })
  })
})

describe('library.view', () => {
  it('names the catalogue’s options, in its order', () => {
    const entry = catalog.entries.find(e => e.key === 'library.view')
    expect(entry && 'options' in entry ? entry.options : null).toEqual([...GAMES_VIEWS])
    expect(isGamesView('list')).toBe(true)
    expect(isGamesView('rows')).toBe(false)
  })
})

describe('resumeTarget', () => {
  const point = (none: boolean) => ({ slot: 2, thumbUrl: null, none })
  const recent = [game('last', { lastPlayed: 9 }), game('before', { lastPlayed: 5 })]

  it('is the game played last when it has somewhere to pick up from', () => {
    expect(resumeTarget(recent, { last: point(false), before: point(false) })).toBe('last')
  })

  it('is no game while its save is not known, without one, or with no game played', () => {
    expect(resumeTarget(recent, {})).toBeNull()
    expect(resumeTarget(recent, { last: point(true), before: point(false) })).toBeNull()
    expect(resumeTarget([], {})).toBeNull()
  })

  it('is no game when the one played last cannot start', () => {
    const gone = [game('last', { lastPlayed: 9, available: false })]
    expect(resumeTarget(gone, { last: point(false) })).toBeNull()
  })
})

describe('coverKinds', () => {
  it('offers the ROM icon only when the ROM has one', () => {
    expect(coverKinds({ hasIcon: false })).toEqual(['generated', 'user'])
    expect(coverKinds({ hasIcon: true })).toEqual(['generated', 'icon', 'user'])
  })
})
