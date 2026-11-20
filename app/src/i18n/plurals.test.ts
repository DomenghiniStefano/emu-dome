import { describe, expect, it, vi } from 'vitest'
import { i18n } from '@/i18n'

vi.mock('@/emu', () => ({ emu: {} }))

// Counts read «1 shortcut», never «1 shortcuts» (vue-i18n plural forms, both languages).
const COUNTS = [
  'settings.pad.combos.value.count',
  'settings.library.hidden.value.count',
  'settings.cloud.sgames.value.count',
]

describe('count messages', () => {
  it.each(['en', 'it'] as const)('have a singular and a plural in %s', locale => {
    const { t } = i18n.global
    i18n.global.locale.value = locale
    for (const key of COUNTS) {
      const one = t(key, { n: 1, size: '1 MB' }, 1)
      const two = t(key, { n: 2, size: '1 MB' }, 2)
      expect(one).toMatch(/^1 /)
      expect(two).toMatch(/^2 /)
      expect(one).not.toBe(two.replace(/^2/, '1'))
    }
  })
})
