// The text a date and time field holds (`YYYY-MM-DDTHH:MM`, a local time to the minute: the
// settings engine's `Kind::DateTime`) and the `@internationalized/date` value Reka's date field
// edits. Pure.
import { CalendarDateTime } from '@internationalized/date'

const TEXT = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})$/

/** The value of `text`, or null when it is not a date and time that exists. */
export function parseDateTimeText(text: string): CalendarDateTime | null {
  const m = TEXT.exec(text)
  if (!m) return null
  const [year = 0, month = 0, day = 0, hour = 0, minute = 0] = m.slice(1).map(Number)
  if (hour > 23 || minute > 59) return null
  const value = new CalendarDateTime(year, month, day, hour, minute)
  // CalendarDateTime constrains what does not exist (31 April → 30 April): refuse it instead.
  const same =
    value.year === year &&
    value.month === month &&
    value.day === day &&
    value.hour === hour &&
    value.minute === minute
  return same ? value : null
}

const two = (n: number) => String(n).padStart(2, '0')

/** The text of a value, to the minute (seconds dropped). */
export function dateTimeText(value: {
  year: number
  month: number
  day: number
  hour: number
  minute: number
}): string {
  const year = String(value.year).padStart(4, '0')
  return `${year}-${two(value.month)}-${two(value.day)}T${two(value.hour)}:${two(value.minute)}`
}
