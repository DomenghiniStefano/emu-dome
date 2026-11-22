import { describe, expect, it, vi } from 'vitest'
import type { SettingRow } from '@/emu'
import { askBeforeDelete } from './askBeforeDelete'

vi.mock('@/lib/settingsModel', () => ({ readSetting: vi.fn() }))

const row = (value: SettingRow['value']): SettingRow => ({
  key: 'general.confirmdel',
  value,
  source: 'global',
  changed: value === false,
})

describe('askBeforeDelete', () => {
  it('asks while general.confirmdel is on, and skips the question when it is off', async () => {
    expect(await askBeforeDelete(async () => row(true))).toBe(true)
    expect(await askBeforeDelete(async () => row(false))).toBe(false)
  })

  it('asks when the setting has no value or cannot be read', async () => {
    expect(await askBeforeDelete(async () => row(null))).toBe(true)
    expect(await askBeforeDelete(async () => undefined)).toBe(true)
    expect(
      await askBeforeDelete(async () => {
        throw new Error('settingsReadFailed')
      }),
    ).toBe(true)
  })
})
