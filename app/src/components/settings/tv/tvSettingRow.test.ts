import { describe, expect, it, vi } from 'vitest'
import type { SettingEntry, SettingKind } from '@/emu'
import {
  MAX_IN_PLACE,
  opensSheet,
  scopeChoiceAt,
  scopeChoices,
  stepOption,
  stepSlider,
  toggled,
  tvEdit,
} from './tvSettingRow'

// The bridge reads Tauri's globals when it loads; these functions never call it.
vi.mock('@/emu', () => ({ emu: {} }))

const entry = (kind: SettingKind, over: Partial<SettingEntry> = {}): SettingEntry => ({
  key: 'video.fps',
  page: 'video',
  kind,
  default: null,
  levels: ['default', 'global'],
  perDevice: false,
  platform: 'all',
  phase: 0,
  hiddenOptions: [],
  synonyms: [],
  mostUsed: false,
  needsNativeVideo: false,
  wired: true,
  unsupportedConsoles: [],
  consoles: ['gb', 'gbc', 'gba', 'nds', '3ds'],
  ...over,
})
const choice = (n: number, moreAtRuntime = false): SettingKind => ({
  type: 'choice',
  style: 'list',
  options: Array.from({ length: n }, (_, i) => `o${i}`),
  moreAtRuntime,
})

describe('tvEdit: in place, in the sheet, run, or the desk row', () => {
  it('switches and sliders change in place', () => {
    expect(tvEdit(entry({ type: 'toggle' }))).toBe('toggle')
    expect(tvEdit(entry({ type: 'slider', min: 0, max: 10, step: 1, unit: 'percent' }))).toBe(
      'step',
    )
  })

  it('a short choice steps, a long one or one that grows at run time opens the sheet', () => {
    expect(tvEdit(entry(choice(MAX_IN_PLACE)))).toBe('step')
    expect(tvEdit(entry(choice(MAX_IN_PLACE + 1)))).toBe('sheet')
    expect(tvEdit(entry(choice(2, true)))).toBe('sheet')
  })

  it('counts only the options this device offers', () => {
    const long = entry(choice(MAX_IN_PLACE + 1), { hiddenOptions: ['o0'] })
    expect(tvEdit(long)).toBe('step')
  })

  it('a few colours step, many open the sheet; multiple choices and presets open it', () => {
    expect(tvEdit(entry({ type: 'colour', swatches: ['#1', '#2', '#3'] }))).toBe('step')
    const many = Array.from({ length: MAX_IN_PLACE + 1 }, (_, i) => `#${i}`)
    expect(tvEdit(entry({ type: 'colour', swatches: many }))).toBe('sheet')
    expect(tvEdit(entry({ type: 'multi', options: ['a'] }))).toBe('sheet')
    expect(tvEdit(entry({ type: 'apply', options: ['a'] }))).toBe('sheet')
  })

  it('files, actions, binding lists and screen layouts run as on the desk', () => {
    expect(tvEdit(entry({ type: 'file' }))).toBe('run')
    expect(tvEdit(entry({ type: 'action', style: 'button' }))).toBe('run')
    expect(tvEdit(entry({ type: 'bindings', table: 'remap' }))).toBe('run')
    expect(tvEdit(entry({ type: 'screenLayout' }))).toBe('run')
  })

  it('a text, the folders and a kind it does not know keep the desk row', () => {
    expect(tvEdit(entry({ type: 'text' }))).toBe('desk')
    expect(tvEdit(entry({ type: 'folders' }))).toBe('desk')
    expect(tvEdit(entry({ type: 'dateTime' } as unknown as SettingKind))).toBe('desk')
  })

  it('A opens the sheet for every choice and colour, not for a slider or a switch', () => {
    expect(opensSheet(entry(choice(3)))).toBe(true)
    expect(opensSheet(entry(choice(9)))).toBe(true)
    expect(opensSheet(entry({ type: 'colour', swatches: ['#1'] }))).toBe(true)
    expect(opensSheet(entry({ type: 'slider', min: 0, max: 1, step: 1, unit: 'px' }))).toBe(false)
    expect(opensSheet(entry({ type: 'toggle' }))).toBe(false)
    expect(opensSheet(entry({ type: 'file' }))).toBe(false)
  })
})

describe('stepOption', () => {
  const opts = (...ids: string[]) => ids.map(id => ({ id, notYet: id.startsWith('x') }))

  it('steps round the ends', () => {
    expect(stepOption(opts('a', 'b', 'c'), 'b', 1)).toBe('c')
    expect(stepOption(opts('a', 'b', 'c'), 'c', 1)).toBe('a')
    expect(stepOption(opts('a', 'b', 'c'), 'a', -1)).toBe('c')
  })

  it('skips the options not pickable yet', () => {
    expect(stepOption(opts('a', 'x1', 'b'), 'a', 1)).toBe('b')
    expect(stepOption(opts('a', 'x1', 'b'), 'a', -1)).toBe('b')
  })

  it('null when nothing else can be picked', () => {
    expect(stepOption(opts('a'), 'a', 1)).toBeNull()
    expect(stepOption(opts('a', 'x1'), 'a', 1)).toBeNull()
    expect(stepOption([], 'a', 1)).toBeNull()
  })

  it('from a value not listed, ▶ picks the first and ◀ the last', () => {
    expect(stepOption(opts('a', 'b'), '', 1)).toBe('a')
    expect(stepOption(opts('a', 'b'), '', -1)).toBe('b')
  })
})

describe('stepSlider', () => {
  const range = { min: 0, max: 100, step: 10 }

  it('steps within the range, null at the end it moves to', () => {
    expect(stepSlider(50, range, 1)).toBe(60)
    expect(stepSlider(50, range, -1)).toBe(40)
    expect(stepSlider(100, range, 1)).toBeNull()
    expect(stepSlider(0, range, -1)).toBeNull()
  })

  it('lands on the steps from min, and does not drift with decimal steps', () => {
    expect(stepSlider(43, range, 1)).toBe(50)
    expect(stepSlider(0.2, { min: 0, max: 1, step: 0.1 }, 1)).toBe(0.3)
    expect(stepSlider(5, { min: 1, max: 9, step: 2 }, 1)).toBe(7)
  })

  it('null for a range that cannot step', () => {
    expect(stepSlider(1, { min: 0, max: 1, step: 0 }, 1)).toBeNull()
  })
})

describe('the rest', () => {
  it('toggled', () => {
    expect(toggled(true)).toBe(false)
    expect(toggled(false)).toBe(true)
    expect(toggled(null)).toBe(true)
  })

  it('scopeChoices: global, the consoles when the page has the level, the game when offered', () => {
    expect(scopeChoices(['global'], ['gba', 'nds'])).toEqual([{ id: 'global', level: 'global' }])
    expect(scopeChoices(['global', 'console', 'game'], ['gba']).map(c => c.id)).toEqual([
      'global',
      'gba',
      'game',
    ])
    expect(scopeChoiceAt('console', 'nds')).toBe('nds')
    expect(scopeChoiceAt('game', 'nds')).toBe('game')
    expect(scopeChoiceAt('global', 'nds')).toBe('global')
  })
})
