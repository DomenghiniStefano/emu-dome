// Writing a setting at «Tutti i giochi» from the Controls section; a failure goes to `onError`
// as text, as everywhere in the section.
import { emu, type SettingValue } from '../../emu'
import { errorText } from '../../i18n/labels'

const GLOBAL = { level: 'global' } as const

export function useGlobalSettings(onError: (text: string) => void) {
  async function set(key: string, value: SettingValue) {
    try {
      await emu.settings.set(key, GLOBAL, value)
    } catch (e) {
      onError(errorText(e))
    }
  }
  async function reset(key: string) {
    try {
      await emu.settings.reset(key, GLOBAL)
    } catch (e) {
      onError(errorText(e))
    }
  }
  return { set, reset }
}
