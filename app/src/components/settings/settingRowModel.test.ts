import { describe, expect, it, vi } from 'vitest'
import type { SettingEntry, SettingRow, SettingsSchema } from '@/emu'
import { SURROUND_CARDS } from '@/lib/surround'
import {
  cardPreview,
  chosenFileName,
  chosenList,
  descNotes,
  hasRuntimeOptions,
  offeredOptions,
  onlyHere,
  rowState,
  rowValue,
  runtimeOptions,
  swatchKey,
  toggleChosen,
} from './settingRowModel'

// The bridge reads Tauri's globals when it loads; these functions never call it.
vi.mock('@/emu', () => ({ emu: {} }))

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
  wired: true,
  unsupportedConsoles: [],
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

describe('onlyHere', () => {
  const phone: SettingsSchema = { ...schema, device: 'phone' }
  it('badges a per-device row with this device', () => {
    expect(onlyHere(entry({ perDevice: true }), phone)).toBe('phone')
    expect(onlyHere(entry({ perDevice: false }), phone)).toBeNull()
  })

  it('never badges a row this device does not have (library.watch on a phone)', () => {
    const desktopOnly = entry({ perDevice: true, platform: 'desktop' })
    expect(onlyHere(desktopOnly, phone)).toBeNull()
    expect(onlyHere(desktopOnly, schema)).toBe('desktop')
  })
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

  it('locks a row on a console whose emulator has no equivalent, and only there', () => {
    const frameskip = entry({ key: 'perf.frameskip', unsupportedConsoles: ['nds', '3ds'] })
    const at = (console: 'gba' | 'nds' | null) =>
      rowState(frameskip, row('global'), schema, 'console', true, console)
    expect(at('nds')).toMatchObject({ reason: 'settingsUi.lockedConsole', disabled: true })
    expect(at('gba').reason).toBeNull()
    expect(at(null).reason).toBeNull()
  })

  it('disables a row with no backend yet without a lock reason', () => {
    // `wired: false` comes from the catalogue (Rust): every action, and values nothing reads.
    const action = entry({
      key: 'about.log',
      kind: { type: 'action', style: 'button' },
      wired: false,
    })
    const unread = entry({ key: 'video.fps', wired: false })
    for (const e of [action, unread])
      expect(rowState(e, undefined, schema, 'global', false)).toMatchObject({
        reason: null,
        notYet: true,
        disabled: true,
      })
    expect(rowState(entry(), undefined, schema, 'global', false).notYet).toBe(false)
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

describe('descNotes', () => {
  const locked = { reason: 'settingsUi.lockedGlobal', notYet: false }

  it('puts the lock reason before the row’s words, unless the page already says it once', () => {
    expect(descNotes(locked, false)).toEqual({ reason: 'settingsUi.lockedGlobal', notYet: false })
    expect(descNotes(locked, true)).toEqual({ reason: null, notYet: false })
    const desktop = { reason: 'settingsUi.lockedDesktop', notYet: false }
    expect(descNotes(desktop, true).reason).toBe('settingsUi.lockedDesktop')
  })

  it('says «Arriva…» only for a row nothing locks', () => {
    expect(descNotes({ reason: null, notYet: true }, false)).toEqual({ reason: null, notYet: true })
    expect(descNotes({ ...locked, notYet: true }, false).notYet).toBe(false)
    // The lock is still there when the page says it: the row does not claim it is coming either.
    expect(descNotes({ ...locked, notYet: true }, true)).toEqual({ reason: null, notYet: false })
  })
})

describe('offeredOptions', () => {
  it('drops the options of another device, and has none for a row that is not a choice', () => {
    const choice = entry({
      kind: {
        type: 'choice',
        style: 'cards',
        options: ['black', 'blur', 'image'],
        moreAtRuntime: false,
      },
      hiddenOptions: ['blur'],
    })
    expect(offeredOptions(choice)).toEqual(['black', 'image'])
    expect(offeredOptions(entry({ kind: { type: 'multi', options: ['a', 'b'] } }))).toEqual([
      'a',
      'b',
    ])
    expect(offeredOptions(entry())).toEqual([])
  })
})

describe('runtimeOptions', () => {
  it('adds the devices found after the fixed options, each once', () => {
    const out = entry({
      key: 'audio.out',
      kind: { type: 'choice', style: 'list', options: ['system'], moreAtRuntime: true },
    })
    expect(hasRuntimeOptions(out)).toBe(true)
    expect(hasRuntimeOptions(entry())).toBe(false)
    expect(
      runtimeOptions(['system'], ['Speakers', '', 'system', 'Headphones', 'Speakers']),
    ).toEqual(['Speakers', 'Headphones'])
  })
})
