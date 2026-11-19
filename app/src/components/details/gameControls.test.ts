import { describe, expect, it, vi } from 'vitest'
import type { SettingRow } from '../../emu'
import { hasOwnRemap, hasTouchScreen, pointerLabel, type PadRows } from './gameControls'
import type { Translator } from '../../i18n/message'

// The bridge talks to Tauri as it loads; these functions never call it.
vi.mock('../../emu', () => ({ emu: {} }))

const row = (
  key: string,
  value: SettingRow['value'],
  source: SettingRow['source'],
): SettingRow => ({ key, value, source, changed: true })

const i18n: Translator = {
  t: key => `[${key}]`,
  te: key => key.endsWith('.option.mouse') || key.endsWith('.option.stick'),
}

describe('hasTouchScreen', () => {
  it('is true for the two-screen consoles only', () => {
    expect(hasTouchScreen('nds')).toBe(true)
    expect(hasTouchScreen('3ds')).toBe(true)
    expect(hasTouchScreen('gba')).toBe(false)
  })
})

describe('hasOwnRemap', () => {
  it('is true only when pad.remap is stored at the game level', () => {
    expect(hasOwnRemap({ 'pad.remap': row('pad.remap', [], 'game') })).toBe(true)
    expect(hasOwnRemap({ 'pad.remap': row('pad.remap', [], 'console') })).toBe(false)
    expect(hasOwnRemap({})).toBe(false)
  })
})

describe('pointerLabel', () => {
  it('names the chosen option', () => {
    const rows: PadRows = { 'pad.pointer': row('pad.pointer', 'stick', 'game') }
    expect(pointerLabel(rows, i18n)).toBe('[settings.pad.pointer.option.stick]')
  })

  it('falls back to the mouse, and to the id without a translation', () => {
    expect(pointerLabel({}, i18n)).toBe('[settings.pad.pointer.option.mouse]')
    const rows: PadRows = { 'pad.pointer': row('pad.pointer', 'pen', 'game') }
    expect(pointerLabel(rows, i18n)).toBe('pen')
  })
})
