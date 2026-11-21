// Writing a setting at «Tutti i giochi» (Controls, the first run); a failure goes to `onError`.
import { emu, type SettingKey, type SettingValue } from '@/emu'
import type { ErrorSink } from '@/i18n/labels'
import { GLOBAL } from '@/lib/settingsModel'

export function useGlobalSettings(onError: ErrorSink) {
  async function set(key: SettingKey, value: SettingValue) {
    try {
      await emu.settings.set(key, GLOBAL, value)
    } catch (e) {
      onError(e)
    }
  }
  async function reset(key: SettingKey) {
    try {
      await emu.settings.reset(key, GLOBAL)
    } catch (e) {
      onError(e)
    }
  }
  return { set, reset }
}
