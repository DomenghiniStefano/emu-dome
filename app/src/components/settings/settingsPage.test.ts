import { describe, expect, it, vi } from 'vitest'
import type { LibraryGame, SettingEntry, SettingKey, SettingsSchema } from '@/emu'
import {
  changedCount,
  effectiveLevel,
  lastPlayedConsole,
  offersGameLevel,
  pageSections,
  RESULTS,
  scopeAt,
  scopeConsoles,
  scopeLevels,
  searchSections,
  stepLevel,
  type PageValues,
} from './settingsPage'

// The bridge reads Tauri's globals when it loads; these functions never call it.
vi.mock('@/emu', () => ({ emu: {} }))

const entry = (key: SettingKey, over: Partial<SettingEntry> = {}): SettingEntry => ({
  key,
  page: key.split('.')[0] ?? '',
  kind: { type: 'toggle' },
  default: false,
  levels: ['default', 'global', 'console'],
  perDevice: false,
  platform: 'all',
  phase: 0,
  hiddenOptions: [],
  synonyms: [],
  mostUsed: false,
  needsNativeVideo: false,
  wired: true,
  unsupportedConsoles: [],
  ...over,
})
const surround = entry('video.surround', {
  kind: { type: 'choice', style: 'cards', options: ['black', 'image'], moreAtRuntime: false },
  default: 'black',
})
const image = entry('video.surroundImg', {
  kind: { type: 'file' },
  default: '',
  showIf: { key: 'video.surround', values: ['image'], not: false },
})
const fps = entry('video.fps', { synonyms: ['framerate'] })
const later = entry('video.res3d', { phase: 2 })
const theme = entry('general.theme')
const schema: SettingsSchema = {
  device: 'desktop',
  currentPhase: 1,
  pages: ['video', 'general'],
  groups: [],
  sections: [
    { page: 'video', id: 'image', keys: ['video.surround', 'video.surroundImg', 'video.res3d'] },
    { page: 'video', id: 'window', keys: ['video.fps'] },
    { page: 'general', id: 'look', keys: ['general.theme'] },
  ],
  entries: [surround, image, fps, later, theme],
}
const byKey = new Map(schema.entries.map(e => [e.key, e]))
const rows = (over: Record<string, Partial<PageValues[string]>>): PageValues =>
  Object.fromEntries(
    Object.entries(over).map(([key, r]) => [
      key,
      { key: key as SettingKey, value: false, source: 'default', changed: false, ...r },
    ]),
  )
const keys = (secs: { entries: SettingEntry[] }[]) => secs.map(s => s.entries.map(e => e.key))

describe('pageSections', () => {
  it('shows a row only while its showIf holds, and hides later phases', () => {
    expect(keys(pageSections(schema, 'video', {}, byKey, null))).toEqual([
      ['video.surround'],
      ['video.fps'],
    ])
    const values = { video: rows({ 'video.surround': { value: 'image' } }) }
    expect(keys(pageSections(schema, 'video', values, byKey, null))[0]).toEqual([
      'video.surround',
      'video.surroundImg',
    ])
  })

  it('keeps only the rows changed at the level, dropping empty sections', () => {
    const values = { video: rows({ 'video.fps': { source: 'console' } }) }
    const secs = pageSections(schema, 'video', values, byKey, 'console')
    expect(secs.map(s => s.id)).toEqual(['window'])
  })
})

describe('searchSections', () => {
  const words = (e: SettingEntry) => `${e.key} ${e.synonyms.join(' ')}`

  it('finds by synonym, accents and case aside, one section per page', () => {
    const secs = searchSections(schema, '  FRÁMERATE ', {}, byKey, words)
    expect(secs).toEqual([{ id: RESULTS, page: 'video', entries: [fps] }])
  })

  it('finds nothing for an empty query or a word nobody has', () => {
    expect(searchSections(schema, '  ', {}, byKey, words)).toEqual([])
    expect(searchSections(schema, 'zzzz', {}, byKey, words)).toEqual([])
  })
})

describe('changedCount', () => {
  it('counts the shown rows set at the level', () => {
    const values = rows({ 'video.fps': { source: 'global' }, 'video.res3d': { source: 'global' } })
    expect(changedCount(schema, 'video', values, 'global')).toBe(1)
    expect(changedCount(schema, 'video', undefined, 'global')).toBe(0)
  })
})

describe('levels and consoles', () => {
  it('steps round the levels, and not at all with one', () => {
    expect(stepLevel(['global', 'console', 'game'], 'game', 1)).toBe('global')
    expect(stepLevel(['global', 'console'], 'global', -1)).toBe('console')
    expect(stepLevel(['global'], 'global', 1)).toBeNull()
  })

  const game = (console: LibraryGame['console'], lastPlayed: number | null) =>
    ({ console, lastPlayed }) as LibraryGame

  it('offers the consoles with games, plus the open one, in the fixed order', () => {
    const games = [game('nds', null), game('gb', 5)]
    expect(scopeConsoles(games, ['3ds', null])).toEqual(['gb', 'nds', '3ds'])
  })

  it('finds the console played last', () => {
    expect(lastPlayedConsole([game('gb', 5), game('nds', 9), game('gba', null)])).toBe('nds')
    expect(lastPlayedConsole([game('gba', null)])).toBeNull()
  })
})

describe('the scope bar', () => {
  const globalOnly = entry('general.theme', { levels: ['default', 'global'] })
  const perConsole = entry('video.fps')
  const perGame = entry('video.scale', { levels: ['default', 'global', 'game'] })
  const zelda = { level: 'game', console: 'gba', game: 'gba:01' } as const

  it('offers «Solo questo gioco» only for a game, on a page it can change', () => {
    expect(offersGameLevel([perGame], true)).toBe(true)
    expect(offersGameLevel([perGame], false)).toBe(false)
    expect(offersGameLevel([perConsole, globalOnly], true)).toBe(false)
  })

  it('lists the console’s level only with settings per console and a console chosen', () => {
    expect(scopeLevels([globalOnly], 'gba', true)).toEqual(['global'])
    expect(scopeLevels([perConsole], null, false)).toEqual(['global'])
    expect(scopeLevels([perConsole], 'gba', false)).toEqual(['global', 'console'])
    expect(scopeLevels([perConsole, perGame], 'gba', true)).toEqual(['global', 'console', 'game'])
    expect(scopeLevels([perGame], 'gba', true)).toEqual(['global', 'game'])
  })

  it('falls back to «Tutti i giochi» when the page has not the level chosen', () => {
    expect(effectiveLevel(['global', 'console'], 'console')).toBe('console')
    expect(effectiveLevel(['global', 'console'], 'game')).toBe('global')
  })

  it('edits the game’s or the console’s scope only when it has one', () => {
    expect(scopeAt('game', 'gb', zelda)).toEqual(zelda)
    expect(scopeAt('game', 'gb', null)).toEqual({ level: 'global' })
    expect(scopeAt('console', 'gb', zelda)).toEqual({ level: 'console', console: 'gb' })
    expect(scopeAt('console', null, null)).toEqual({ level: 'global' })
    expect(scopeAt('global', 'gb', zelda)).toEqual({ level: 'global' })
  })
})
