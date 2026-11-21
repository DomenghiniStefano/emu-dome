import { describe, expect, it, vi } from 'vitest'
import type { SettingEntry, SettingKey, SettingKind, SettingsSchema } from '@/emu'
import { overrideRows, settingValueText } from './gameOverrides'
import type { Translator } from '@/i18n/message'

// The bridge talks to Tauri as it loads; these functions never call it.
vi.mock('@/emu', () => ({ emu: {} }))

const KNOWN = new Set(['settings.video.filter.option.smooth'])
const i18n: Translator = {
  t: (key, params) => (params ? `${key}${JSON.stringify(params)}` : key),
  te: key => KNOWN.has(key),
}

const entry = (key: SettingKey, kind: SettingKind, phase = 0): SettingEntry => ({
  key,
  page: key.split('.')[0]!,
  kind,
  default: null,
  levels: ['default', 'global', 'console', 'game'],
  perDevice: false,
  platform: 'all',
  phase,
  hiddenOptions: [],
  synonyms: [],
  mostUsed: false,
  needsNativeVideo: false,
})

const filter = entry('video.filter', {
  type: 'choice',
  style: 'list',
  options: ['smooth', 'sharp'],
  moreAtRuntime: false,
})

describe('settingValueText', () => {
  it('names a toggle on or off', () => {
    const e = entry('general.bgpause', { type: 'toggle' })
    expect(settingValueText(e, true, i18n)).toBe('details.settings.on')
    expect(settingValueText(e, false, i18n)).toBe('details.settings.off')
  })

  it('names a choice by its option, or by its id without a translation', () => {
    expect(settingValueText(filter, 'smooth', i18n)).toBe('settings.video.filter.option.smooth')
    expect(settingValueText(filter, 'sharp', i18n)).toBe('sharp')
  })

  it('lists the options of a multiple choice', () => {
    const e = entry('video.filter', { type: 'multi', options: ['smooth', 'sharp'] })
    expect(settingValueText(e, ['smooth', 'sharp'], i18n)).toBe(
      'settings.video.filter.option.smooth, sharp',
    )
  })

  it('puts a slider value in its unit', () => {
    const e = entry('a11y.textsize', {
      type: 'slider',
      min: 80,
      max: 150,
      step: 10,
      unit: 'percent',
    })
    expect(settingValueText(e, 120, i18n)).toBe('settingsUi.unit.percent{"v":120}')
  })

  it('says nothing of a binding list without its custom label', () => {
    const e = entry('pad.remap', { type: 'bindings', table: 'remap' })
    expect(settingValueText(e, 'x', i18n)).toBe('')
  })

  it('joins folders', () => {
    expect(settingValueText(entry('library.folders', { type: 'folders' }), ['a', 'b'], i18n)).toBe(
      'a, b',
    )
  })
})

describe('overrideRows', () => {
  const schema = {
    device: 'desktop',
    currentPhase: 1,
    pages: [],
    groups: [],
    sections: [],
    entries: [filter, entry('later.thing' as SettingKey, { type: 'toggle' }, 2)],
  } as unknown as SettingsSchema

  it('names each override with its page and both values', () => {
    const rows = overrideRows(
      [{ key: 'video.filter', page: 'video', value: 'sharp', base: 'smooth' }],
      schema,
      i18n,
    )
    expect(rows).toEqual([
      {
        key: 'video.filter',
        name: 'settings.video.filter.name',
        where: 'settings.pages.video.name',
        value: 'sharp',
        base: 'settings.video.filter.option.smooth',
      },
    ])
  })

  it('keeps a null base when the console has nothing set', () => {
    const [row] = overrideRows(
      [{ key: 'video.filter', page: 'video', value: 'sharp', base: null }],
      schema,
      i18n,
    )
    expect(row?.base).toBeNull()
  })

  it('leaves out keys the schema does not know or does not show yet', () => {
    const rows = overrideRows(
      [
        // Keys of another version of the catalogue: not in SettingKey on purpose.
        { key: 'gone.key' as SettingKey, page: 'gone', value: true, base: null },
        { key: 'later.thing' as SettingKey, page: 'later', value: true, base: null },
      ],
      schema,
      i18n,
    )
    expect(rows).toEqual([])
  })

  it('has nothing before the schema is loaded', () => {
    expect(
      overrideRows([{ key: 'video.filter', page: 'video', value: 'x', base: null }], null, i18n),
    ).toEqual([])
  })
})
