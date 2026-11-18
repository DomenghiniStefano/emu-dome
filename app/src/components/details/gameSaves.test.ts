import { describe, expect, it, vi } from 'vitest'
import type { SettingRow } from '../../emu'
import { quickSlotCount } from './gameSaves'

// The bridge talks to Tauri as it loads; these functions never call it.
vi.mock('../../emu', () => ({ emu: {} }))

const row = (key: string, value: SettingRow['value']): SettingRow => ({
  key,
  value,
  source: 'global',
  changed: false,
})

describe('quickSlotCount', () => {
  it('reads saves.slots', () => {
    expect(quickSlotCount([row('saves.other', 3), row('saves.slots', 4)])).toBe(4)
  })

  it('falls back to 10 without the setting, without a value or with a non-number', () => {
    expect(quickSlotCount([])).toBe(10)
    expect(quickSlotCount([row('saves.slots', null)])).toBe(10)
    expect(quickSlotCount([row('saves.slots', 'many')])).toBe(10)
    expect(quickSlotCount([row('saves.slots', 0)])).toBe(10)
  })
})
