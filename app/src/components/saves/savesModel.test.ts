import { describe, expect, it } from 'vitest'
import type { LibraryGame, SavesSummary } from '../../emu'
import {
  emptyTextKey,
  fileStem,
  filterByTitle,
  gamesWithSaves,
  hasSaves,
  matchImportTarget,
  sortByTitle,
} from './savesModel'

const game = (title: string, path = `C:/Games/${title}.gba`) => ({ title, path }) as LibraryGame
const summary = (path: string, s: Partial<SavesSummary> = {}): SavesSummary => ({
  path,
  inGame: false,
  quickCount: 0,
  bytes: 0,
  lastChange: null,
  broken: false,
  ...s,
})

describe('hasSaves', () => {
  it('counts an in-game save, a quick save or a broken save', () => {
    expect(hasSaves(summary('a', { inGame: true }))).toBe(true)
    expect(hasSaves(summary('a', { quickCount: 1 }))).toBe(true)
    expect(hasSaves(summary('a', { broken: true }))).toBe(true)
  })

  it('is false for nothing saved or no summary', () => {
    expect(hasSaves(summary('a'))).toBe(false)
    expect(hasSaves(undefined)).toBe(false)
  })
})

describe('gamesWithSaves', () => {
  it('keeps the games with saves, the most recently changed first', () => {
    const [a, b, c] = [game('A'), game('B'), game('C')]
    const summaries = {
      [a.path]: summary(a.path, { inGame: true, lastChange: 10 }),
      [b.path]: summary(b.path),
      [c.path]: summary(c.path, { quickCount: 2, lastChange: 20 }),
    }
    expect(gamesWithSaves([a, b, c], summaries)).toEqual([c, a])
  })

  it('puts a game whose last change is unknown last', () => {
    const [a, b] = [game('A'), game('B')]
    const summaries = {
      [a.path]: summary(a.path, { broken: true }),
      [b.path]: summary(b.path, { inGame: true, lastChange: 5 }),
    }
    expect(gamesWithSaves([a, b], summaries)).toEqual([b, a])
  })
})

describe('filterByTitle', () => {
  const games = [game('Pocket Quest'), game('Star Pilot')]

  it('keeps every game for an empty or blank query', () => {
    expect(filterByTitle(games, '')).toEqual(games)
    expect(filterByTitle(games, '   ')).toEqual(games)
  })

  it('matches part of the title, ignoring case and outer spaces', () => {
    expect(filterByTitle(games, ' PILOT ')).toEqual([games[1]])
    expect(filterByTitle(games, 'zzz')).toEqual([])
  })
})

describe('sortByTitle', () => {
  it('sorts by title without changing the input', () => {
    const games = [game('b'), game('a')]
    expect(sortByTitle(games).map(g => g.title)).toEqual(['a', 'b'])
    expect(games.map(g => g.title)).toEqual(['b', 'a'])
  })
})

describe('fileStem', () => {
  it('drops the folder and the extension, in lower case', () => {
    expect(fileStem('C:\\Saves\\Pocket Quest.SAV')).toBe('pocket quest')
    expect(fileStem('/home/me/star.pilot.dsv')).toBe('star.pilot')
    expect(fileStem('plain')).toBe('plain')
  })
})

describe('matchImportTarget', () => {
  const byFile = game('Pocket Quest Advance', 'C:/Roms/pqa.gba')
  const byTitle = game('Star Pilot', 'C:/Roms/sp-eu.gba')

  it('prefers the ROM with the same file name', () => {
    expect(matchImportTarget('PQA.sav', [byTitle, byFile])).toBe(byFile)
  })

  it('falls back to the same title', () => {
    expect(matchImportTarget('star pilot.dsv', [byFile, byTitle])).toBe(byTitle)
  })

  it('finds nothing when neither matches', () => {
    expect(matchImportTarget('other.sav', [byFile, byTitle])).toBeNull()
  })
})

describe('emptyTextKey', () => {
  it('follows the auto quick save interval, five minutes by default', () => {
    expect(emptyTextKey('1min')).toBe('saves.empty.text1min')
    expect(emptyTextKey('never')).toBe('saves.empty.textNever')
    expect(emptyTextKey('5min')).toBe('saves.empty.text5min')
    expect(emptyTextKey('anything else')).toBe('saves.empty.text5min')
  })
})
