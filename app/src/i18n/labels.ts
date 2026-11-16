// Backend data → text: error codes, speeds, controller names. The backend sends codes
// and values only; every sentence comes from the message files.
import { i18n, t } from './index'
import type { ErrorCode, GamepadEvent, Speed } from '../emu'

const known = (key: string) => i18n.global.te(key)

/** Shape check for an `ErrorCode` coming back from a command or an event. */
export function isErrorCode(e: unknown): e is ErrorCode {
  return typeof e === 'object' && e !== null && typeof (e as { code?: unknown }).code === 'string'
}

/** Readable text for any error: known codes are translated, anything else is shown as a detail. */
export function errorText(e: unknown): string {
  if (isErrorCode(e) && known(`errors.${e.code}`)) {
    const params: Record<string, unknown> = { ...('params' in e ? e.params : {}) }
    if (e.code === 'consoleNotYet') {
      const name = `console.${e.params.console}`
      params.console = known(name) ? t(name) : e.params.console
    }
    return t(`errors.${e.code}`, params)
  }
  const detail = isErrorCode(e) ? e.code : e instanceof Error ? e.message : String(e)
  return t('errors.unexpected', { detail })
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
