import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n } from './index'
import {
  changedWhenLabel,
  daysApart,
  lastPlayedLabel,
  saveWhenLabel,
  errorText,
  settingDateTimeLabel,
  slotDateLabel,
  stateLoadBlockText,
  timeAgoLabel,
} from './labels'
import type { StateLoadBlock } from '@/emu'

// i18n/index.ts follows `general.lang` through the bridge, which reads Tauri's internals.
vi.mock('@/emu', () => ({ emu: { settings: { onChanged: () => Promise.resolve(() => {}) } } }))

const at = (y: number, m: number, d: number, h = 12, min = 0) =>
  new Date(y, m - 1, d, h, min).getTime()
const NOW = at(2026, 11, 19, 10, 0)
const MINUTE = 60_000

beforeEach(() => {
  i18n.global.locale.value = 'en'
})
afterEach(() => {
  i18n.global.locale.value = 'en'
})

describe('daysApart', () => {
  it('counts calendar days, not 24-hour spans', () => {
    expect(daysApart(at(2026, 11, 19, 0, 1), NOW)).toBe(0)
    expect(daysApart(at(2026, 11, 18, 23, 59), NOW)).toBe(1)
    expect(daysApart(at(2026, 11, 12, 10), NOW)).toBe(7)
  })
  it('crosses a month and a year', () => {
    expect(daysApart(at(2026, 10, 31), at(2026, 11, 1))).toBe(1)
    expect(daysApart(at(2025, 12, 31), at(2026, 1, 1))).toBe(1)
  })
})

describe('lastPlayedLabel', () => {
  it('says the day, then the week, then the month', () => {
    expect(lastPlayedLabel(at(2026, 11, 19, 8), NOW)).toBe('today')
    expect(lastPlayedLabel(at(2026, 11, 18, 23), NOW)).toBe('yesterday')
    expect(lastPlayedLabel(at(2026, 11, 16), NOW)).toBe('3 days ago')
    expect(lastPlayedLabel(at(2026, 11, 10), NOW)).toBe('last week')
    expect(lastPlayedLabel(at(2026, 10, 2), NOW)).toBe('October')
    expect(lastPlayedLabel(at(2025, 10, 2), NOW)).toBe('October 2025')
  })
})

describe('saveWhenLabel', () => {
  it('says today and yesterday with the time, then the day', () => {
    expect(saveWhenLabel(at(2026, 11, 19, 8, 12), NOW)).toContain('08:12')
    expect(saveWhenLabel(at(2026, 11, 19, 8, 12), NOW)).toMatch(/today/i)
    expect(saveWhenLabel(at(2026, 11, 18, 22, 41), NOW)).toMatch(/yesterday/i)
    expect(saveWhenLabel(at(2026, 11, 12, 19, 2), NOW)).toContain('Nov 12')
    expect(saveWhenLabel(at(2025, 11, 12, 19, 2), NOW)).toContain('2025')
  })
})

describe('changedWhenLabel', () => {
  it('says today or yesterday, then the day', () => {
    expect(changedWhenLabel(at(2026, 11, 19, 8), NOW)).toBe('today')
    expect(changedWhenLabel(at(2026, 11, 18, 8), NOW)).toBe('yesterday')
    expect(changedWhenLabel(at(2026, 11, 12, 8), NOW)).toBe('Nov 12')
    expect(changedWhenLabel(at(2025, 11, 12, 8), NOW)).toBe('Nov 12, 2025')
  })
})

describe('settingDateTimeLabel', () => {
  it('writes a date and time setting as a local date and time', () => {
    const text = settingDateTimeLabel('2004-01-02T13:05')
    expect(text).toContain('2004')
    expect(text).toContain('Jan 2')
    expect(text).toMatch(/1:05/)
  })
  it('shows a text that is not one as it is', () => {
    expect(settingDateTimeLabel('soon')).toBe('soon')
  })
})

describe('slotDateLabel', () => {
  it('writes the short date and time of the quick menu’s tiles', () => {
    expect(slotDateLabel(at(2026, 11, 18, 9, 30))).toMatch(/11\/18\/26/)
  })
})

describe('timeAgoLabel', () => {
  it('counts minutes under an hour', () => {
    expect(timeAgoLabel(NOW - 2 * MINUTE, NOW)).toBe('2 minutes ago')
    expect(timeAgoLabel(NOW - 59 * MINUTE, NOW)).toBe('59 minutes ago')
  })
  it('says this minute under half a minute', () => {
    expect(timeAgoLabel(NOW - 10_000, NOW)).toBe('this minute')
  })
  it('counts hours from the hour on', () => {
    expect(timeAgoLabel(NOW - 60 * MINUTE, NOW)).toBe('1 hour ago')
    expect(timeAgoLabel(NOW - 180 * MINUTE, NOW)).toBe('3 hours ago')
  })
  it('speaks the interface language', () => {
    i18n.global.locale.value = 'it'
    expect(timeAgoLabel(NOW - 2 * MINUTE, NOW)).toBe('2 minuti fa')
  })
})

describe('stateLoadBlockText', () => {
  // Every reason Rust can send (a Record: typecheck fails when the generated union grows).
  const REASONS: Record<StateLoadBlock, true> = { vulkanNoTitleId: true }
  const reasons = Object.keys(REASONS) as StateLoadBlock[]

  it('has a text for every reason, in both languages', () => {
    for (const locale of ['en', 'it'] as const) {
      i18n.global.locale.value = locale
      for (const reason of reasons) {
        expect(i18n.global.te(`flow.loadBlock.${reason}`, locale)).toBe(true)
      }
    }
  })
  it('is what the refusal says', () => {
    for (const reason of reasons) {
      const refusal = { code: 'stateLoadUnsupported', params: { reason } }
      expect(errorText(refusal)).toBe(stateLoadBlockText(reason))
    }
  })
})
