import { describe, expect, it, vi } from 'vitest'
import type { ConsoleId, LibraryGame } from '@/emu'
import {
  SHELF_PAGE,
  TV_CONTINUE_COUNT,
  firstShelfLimit,
  heroOverline,
  nearestCover,
  shelfLimitAfter,
  shelfTitle,
  tvShelves,
} from './tvShelves'

// emu.ts reaches for the Tauri bridge when it loads; these functions need only its constant.
vi.mock('@/emu', () => ({ AUTO_SLOT: 0 }))

const game = (
  id: string,
  console: ConsoleId,
  over: Partial<Pick<LibraryGame, 'lastPlayed' | 'favourite' | 'hidden'>> = {},
): LibraryGame =>
  ({
    id,
    console,
    title: id,
    lastPlayed: null,
    favourite: false,
    hidden: false,
    ...over,
  }) as LibraryGame

describe('tvShelves', () => {
  const games = [
    game('Zeta', 'gba', { lastPlayed: 10, favourite: true }),
    game('Alpha', 'nds', { lastPlayed: 30 }),
    game('Beta', 'gba'),
    game('Hidden', 'gb', { lastPlayed: 50, favourite: true, hidden: true }),
  ]

  it('stacks played last, favourites, then one shelf per console present', () => {
    const shelves = tvShelves(games)
    expect(shelves.map(s => s.id)).toEqual(['continue', 'favourites', 'gba', 'nds'])
    expect(shelves[0]?.games.map(g => g.id)).toEqual(['Alpha', 'Zeta'])
    expect(shelves[2]?.games.map(g => g.id)).toEqual(['Beta', 'Zeta'])
  })

  it('leaves out hidden games and the shelves left empty', () => {
    const shelves = tvShelves([game('Only', 'gb', { hidden: true }), game('New', 'gbc')])
    expect(shelves.map(s => s.id)).toEqual(['gbc'])
  })

  it('shows one tile per family, but «Continua a giocare» the versions played (#102)', () => {
    const us = game('us', 'nds', { lastPlayed: 5, favourite: true })
    const eu = game('eu', 'nds', { lastPlayed: 9, favourite: true })
    const shelves = tvShelves([us, eu], new Set([us.id]))
    expect(shelves.map(s => [s.id, s.games.map(g => g.id)])).toEqual([
      ['continue', ['eu', 'us']],
      ['favourites', ['eu']],
      ['nds', ['eu']],
    ])
  })

  it('keeps at most the TV count in «Continua a giocare»', () => {
    const many = Array.from({ length: 20 }, (_, i) => game(`g${i}`, 'gb', { lastPlayed: i }))
    expect(tvShelves(many)[0]?.games).toHaveLength(TV_CONTINUE_COUNT)
  })
})

describe('texts', () => {
  it('names each shelf', () => {
    expect(shelfTitle('continue')).toBe('library.continue.title')
    expect(shelfTitle('favourites')).toBe('tv.shelf.favourites')
    expect(shelfTitle('3ds')).toBe('console.3ds')
  })

  it('says «Continua a giocare» over a game played, its console over a new one', () => {
    expect(heroOverline({ console: 'nds', lastPlayed: 1 })).toEqual({
      key: 'library.continue.title',
    })
    expect(heroOverline({ console: 'nds', lastPlayed: null })).toEqual({ key: 'console.nds' })
  })
})

describe('shelf window', () => {
  const shelf = {
    id: 'gba' as const,
    games: Array.from({ length: 100 }, (_, i) => game(`g${i}`, 'gba')),
  }

  it('renders a page, or up to the game it comes back to', () => {
    expect(firstShelfLimit(shelf, null)).toBe(SHELF_PAGE)
    expect(firstShelfLimit(shelf, 'g3')).toBe(SHELF_PAGE)
    expect(firstShelfLimit(shelf, 'g70')).toBe(76)
  })

  it('renders the next page as the focus comes near the end, never past the total', () => {
    expect(shelfLimitAfter(30, 10, 100)).toBe(30)
    expect(shelfLimitAfter(30, 25, 100)).toBe(60)
    expect(shelfLimitAfter(90, 88, 100)).toBe(100)
    expect(shelfLimitAfter(100, 99, 100)).toBe(100)
  })
})

describe('nearestCover', () => {
  it('lands on the cover nearest the one left', () => {
    expect(nearestCover([100, 300, 500], 320)).toBe(1)
    expect(nearestCover([100, 300, 500], 900)).toBe(2)
    expect(nearestCover([], 10)).toBeNull()
  })
})
