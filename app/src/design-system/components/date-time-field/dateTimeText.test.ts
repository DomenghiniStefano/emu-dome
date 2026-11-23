import { describe, expect, it } from 'vitest'
import { dateTimeText, parseDateTimeText } from './dateTimeText'

describe('parseDateTimeText', () => {
  it('reads a local date and time to the minute', () => {
    const v = parseDateTimeText('2024-02-29T07:05')
    expect(v && [v.year, v.month, v.day, v.hour, v.minute]).toEqual([2024, 2, 29, 7, 5])
  })
  it('refuses a date that does not exist instead of moving it', () => {
    for (const bad of [
      '2023-02-29T07:05',
      '2024-04-31T00:00',
      '2024-13-01T00:00',
      '2024-01-01T24:00',
      '2024-01-01T10:60',
      '2024-01-01 10:00',
      '2024-01-01T10:00:00',
      '',
    ])
      expect(parseDateTimeText(bad), bad).toBeNull()
  })
})

describe('dateTimeText', () => {
  it('writes the settings engine’s form, seconds dropped, and reads back the same', () => {
    expect(dateTimeText({ year: 2004, month: 1, day: 2, hour: 3, minute: 4 })).toBe(
      '2004-01-02T03:04',
    )
    const v = parseDateTimeText('2099-12-31T23:59')
    expect(v && dateTimeText(v)).toBe('2099-12-31T23:59')
  })
})
