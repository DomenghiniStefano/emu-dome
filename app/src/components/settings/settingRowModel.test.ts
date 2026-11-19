import { describe, expect, it, vi } from 'vitest'
import type { SettingEntry, SettingRow, SettingsSchema } from '../../emu'
import { SURROUND_CARDS } from '../../surround'
import {
  cardPreview,
  chosenFileName,
  chosenList,
  rowState,
  rowValue,
  swatchKey,
  toggleChosen,
} from './settingRowModel'

// The bridge reads Tauri's globals when it loads; these functions never call it.
vi.mock('../../emu', () => ({ emu: {} }))

const entry = (over: Partial<SettingEntry> = {}): SettingEntry => ({
  key: 'video.fps',
  page: 'video',
  kind: { type: 'toggle' },
  default: false,
  levels: ['default', 'global', 'console', 'game', 'session'],
  perDevice: false,
  platform: 'all',
  phase: 0,
  hiddenOptions: [],
  synonyms: [],
  mostUsed: false,
  needsNativeVideo: false,
  ...over,
})
const schema: SettingsSchema = {
  device: 'desktop',
  currentPhase: 1,
  pages: ['video'],
  groups: [],
  sections: [],
  entries: [],
}
const row = (source: SettingRow['source'], value: SettingRow['value'] = true): SettingRow => ({
  key: 'video.fps',
  value,
  source,
  changed: source !== 'default',
})

describe('rowState', () => {
  it('marks a value set at the level edited as changed here, resetting to «Tutti i giochi»', () => {
    const s = rowState(entry(), row('console'), schema, 'console', true)
    expect(s).toMatchObject({ changedHere: true, inheritedFrom: null, fallback: 'global' })
  })

  it('resets to the default at «Tutti i giochi»', () => {
    expect(rowState(entry(), row('global'), schema, 'global', true).fallback).toBeNull()
  })

  it('names the level an inherited value comes from, only with a scope bar', () => {
    expect(rowState(entry(), row('global'), schema, 'console', true).inheritedFrom).toBe('global')
    expect(rowState(entry(), row('global'), schema, 'console', false).inheritedFrom).toBeNull()
    expect(rowState(entry(), row('default'), schema, 'console', true).inheritedFrom).toBeNull()
  })

  it('locks a row its level does not accept, and never shows it changed', () => {
    const globalOnly = entry({ levels: ['default', 'global'] })
    const s = rowState(globalOnly, row('console'), schema, 'console', true)
    expect(s).toMatchObject({
      reason: 'settingsUi.lockedGlobal',
      disabled: true,
      changedHere: false,
    })
  })

  it('disables a row with no backend yet without a lock reason', () => {
    const action = entry({ key: 'fx.fxfile', kind: { type: 'action', style: 'button' } })
    expect(rowState(action, undefined, schema, 'global', false)).toMatchObject({
      reason: null,
      notYet: true,
      disabled: true,
    })
  })
})

describe('row values', () => {
  it('falls back to the default without a stored row', () => {
    expect(rowValue(entry({ default: true }), undefined)).toBe(true)
    expect(rowValue(entry(), row('global', false))).toBe(false)
  })

  it('reads a list, anything else as none chosen', () => {
    expect(chosenList(['a', 'b'])).toEqual(['a', 'b'])
    expect(chosenList('a')).toEqual([])
    expect(chosenList(null)).toEqual([])
  })
})

describe('toggleChosen', () => {
  const options = ['protan', 'deutan', 'tritan']

  it('removes a chosen option', () => {
    expect(toggleChosen(['deutan', 'protan'], options, 'deutan')).toEqual(['protan'])
  })

  it('adds an option in the options’ order', () => {
    expect(toggleChosen(['tritan'], options, 'protan')).toEqual(['protan', 'tritan'])
  })
})

describe('card previews and swatches', () => {
  it('takes a background card’s own preview', () => {
    expect(cardPreview('video.surround', 'black', 3)).toBe(SURROUND_CARDS.black)
  })

  it('gives other cards a hue by position, round the list', () => {
    expect(cardPreview('fx.fxpick', 'crt', 0)).toBe(cardPreview('fx.fxpick', 'lcd', 6))
    expect(cardPreview('fx.fxpick', 'crt', 0)).not.toBe(cardPreview('fx.fxpick', 'crt', 1))
  })

  it('names a swatch by its hex without the #', () => {
    expect(swatchKey('general.accent', '#7152f7')).toBe('settings.general.accent.swatch.7152f7')
  })
})

describe('chosenFileName', () => {
  it('keeps the last part of a path, with either separator', () => {
    expect(chosenFileName('C:\\fx\\crt.slangp')).toBe('crt.slangp')
    expect(chosenFileName('/home/me/fx/lcd.slangp')).toBe('lcd.slangp')
    expect(chosenFileName('')).toBe('')
    expect(chosenFileName(null)).toBe('')
    expect(chosenFileName(true)).toBe('')
  })
})
