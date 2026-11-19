// Backend data → text: error codes, speeds, controller names. The backend sends codes
// and values only; every sentence comes from the message files.
import { i18n, t } from './index'
import type { ErrorCode, GamepadEvent, Speed } from '@/emu'

const known = (key: string) => i18n.global.te(key)

/** Shape check for an `ErrorCode` coming back from a command or an event. */
export function isErrorCode(e: unknown): e is ErrorCode {
  return typeof e === 'object' && e !== null && typeof (e as { code?: unknown }).code === 'string'
}

/**
 * Where a composable sends a failure: the error as it was thrown (an `ErrorCode` from the bridge,
 * or anything else). The screen or the toast that shows it turns it into text with `errorText`,
 * so a composable never translates an error itself.
 */
export type ErrorSink = (e: unknown) => void

/** Readable text for any error: known codes are translated, anything else is shown as a detail. */
export function errorText(e: unknown): string {
  if (isErrorCode(e) && known(`errors.${e.code}`)) {
    const params: Record<string, unknown> = { ...('params' in e ? e.params : {}) }
    if (e.code === 'consoleNotYet') {
      const name = `console.${e.params.console}`
      params.console = known(name) ? t(name) : e.params.console
    }
    if (e.code === 'settingInvalidValue') params.name = settingName(e.params.key)
    if (e.code === 'effectNotLoaded') {
      // A built-in effect has its option's name; the user's own file keeps its file name.
      const option = `settings.fx.fxpick.option.${e.params.name}`
      params.name = known(option) ? t(option) : e.params.name
    }
    if (e.code === 'settingLevelRefused') {
      const words = e.params.levels.map(l => t(`settingsUi.levelWord.${l}`))
      params.levels = new Intl.ListFormat(i18n.global.locale.value, {
        type: 'disjunction',
      }).format(words)
    }
    return t(`errors.${e.code}`, params)
  }
  const detail = isErrorCode(e) ? e.code : e instanceof Error ? e.message : String(e)
  return t('errors.unexpected', { detail })
}

/** A setting's name from its key (`settings.<key>.name`), or the key when it has none. */
function settingName(key: string): string {
  const name = `settings.${key}.name`
  return known(name) ? t(name, { controllerName: '' }).trim() : key
}

/** "1×", "0.5×" / "0,5×" in the UI language, or "Max". Empty with no speed. */
export function speedLabel(s: Speed | null): string {
  if (!s) return ''
  if (s.kind === 'max') return t('speed.max')
  const value = new Intl.NumberFormat(i18n.global.locale.value, {
    maximumFractionDigits: 2,
  }).format(s.value)
  return t('speed.factor', { value })
}

/** A controller's name, or a generic one when SDL knows none. */
export function gamepadName(e: GamepadEvent): string {
  return e.name ?? t('toast.gamepadUnnamed')
}

/** Play time on a cover ("14 h", "32 min") or, `long`, in the «Riprendi» hero ("14 h 32 min"). */
export function playTimeLabel(secs: number, long = false): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  if (h === 0) return t('library.time.minutes', { m })
  if (long && m > 0) return t('library.time.hoursMinutes', { h, m })
  return t('library.time.hours', { h })
}

/** A file size for the Saves section: "8 KB", "1.8 MB" / "1,8 MB" in the UI language. */
export function sizeLabel(bytes: number): string {
  const units = ['byte', 'kilobyte', 'megabyte', 'gigabyte'] as const
  let value = bytes
  let i = 0
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024
    i++
  }
  return new Intl.NumberFormat(i18n.global.locale.value, {
    style: 'unit',
    unit: units[i],
    unitDisplay: 'short',
    maximumFractionDigits: value < 10 && i > 0 ? 1 : 0,
  }).format(value)
}

// ---- Dates and times, in the UI language ----
//
// Every date the interface writes is built here. The boards write a date differently per place:
// the library says how long ago a game was played, Saves and the details say the day and the
// time of a save, the quick menu's slot tiles a short date and time, Settings the day only.

const locale = () => i18n.global.locale.value
const format = (ms: number, options: Intl.DateTimeFormatOptions) =>
  new Intl.DateTimeFormat(locale(), options).format(new Date(ms))
const relative = () => new Intl.RelativeTimeFormat(locale(), { numeric: 'auto' })

/** Calendar days from `then` to `now` in local time: 0 the same day, 1 the day before. */
export function daysApart(then: number, now: number): number {
  const day = (ms: number) => {
    const d = new Date(ms)
    return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime()
  }
  return Math.round((day(now) - day(then)) / 86_400_000)
}

const sameYear = (then: number, now: number) =>
  new Date(then).getFullYear() === new Date(now).getFullYear()

/** "12 Nov", with the year when it is not this one. */
const shortDay = (ms: number, now: number) =>
  format(
    ms,
    sameYear(ms, now)
      ? { day: 'numeric', month: 'short' }
      : { day: 'numeric', month: 'short', year: 'numeric' },
  )

/**
 * When a game was last played, as the L01 board writes it: "today", "yesterday", "3 days ago",
 * "last week", then the month ("October", with the year when it is not this one).
 */
export function lastPlayedLabel(ms: number, now = Date.now()): string {
  const days = daysApart(ms, now)
  if (days < 7) return relative().format(-Math.max(0, days), 'day')
  if (days < 14) return relative().format(-1, 'week')
  return format(ms, sameYear(ms, now) ? { month: 'long' } : { month: 'long', year: 'numeric' })
}

/** A full date and time ("17 November 2026 at 08:12"), for sentences. */
export function dateTimeLabel(ms: number): string {
  return format(ms, { dateStyle: 'long', timeStyle: 'short' })
}

/** When a save was written, as the Saves boards write it: "today at 08:12", "yesterday at
 * 22:41", "12 Nov at 19:02" (with the year when it is not this one). */
export function saveWhenLabel(ms: number, now = Date.now()): string {
  const time = format(ms, { hour: '2-digit', minute: '2-digit' })
  const days = daysApart(ms, now)
  if (days === 0) return t('saves.when.today', { time })
  if (days === 1) return t('saves.when.yesterday', { time })
  return t('saves.when.date', { date: shortDay(ms, now), time })
}

/** A quick save slot's date on the quick menu's tiles (G07, G08): short date and time
 * ("11/18/26, 9:30 AM"), as the board's tiles have room for. Saves writes the same save with
 * `saveWhenLabel`: two boards, two formats (Cowork's to bring together). */
export function slotDateLabel(ms: number): string {
  return format(ms, { dateStyle: 'short', timeStyle: 'short' })
}

/** When a setting was last changed, as the T01 board writes it: "today", "yesterday", "12 Nov"
 * (with the year when it is not this one). */
export function changedWhenLabel(ms: number, now = Date.now()): string {
  const days = daysApart(ms, now)
  if (days <= 1) return relative().format(-Math.max(0, days), 'day')
  return shortDay(ms, now)
}

/** «2 minuti fa»: minutes under an hour, then hours (E11, when the last auto quick save was
 * written). */
export function timeAgoLabel(at: number, now = Date.now()): string {
  const minutes = Math.round((at - now) / 60000)
  return Math.abs(minutes) < 60
    ? relative().format(minutes, 'minute')
    : relative().format(Math.round(minutes / 60), 'hour')
}
