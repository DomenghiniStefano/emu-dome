// The settings the quick menu sets for this device at once, not «in prova» (Trello #114,
// `touch.tvib`): they live on the device at «Tutti i giochi» only (no game or console level), so
// there is nothing to try for a game. Read at the Global level, set there, read again on change.
import { ref } from 'vue'
import {
  emu,
  type SettingEntry,
  type SettingKey,
  type SettingRow,
  type SettingsSchema,
  type SettingValue,
} from '@/emu'
import { useSettingsChanged } from '@/composables/useSettingsChanged'
import type { ErrorSink } from '@/i18n/labels'
import { isShown, platformIncludes } from '@/lib/settingsModel'
import { DEVICE_ROWS, type QuickMenuView } from './view'

const GLOBAL = { level: 'global' } as const
const DEVICE_KEYS: SettingKey[] = Object.values(DEVICE_ROWS).flat()
/** The pages they are on: a key's prefix (`touch.tvib` → `touch`). */
const DEVICE_PAGES = [...new Set(DEVICE_KEYS.map(k => k.replace(/\..*/, '')))]

/** The entries `view` sets for the device: those the schema shows on this device. */
export function deviceEntries(view: QuickMenuView, schema: SettingsSchema | null): SettingEntry[] {
  if (!schema) return []
  return (DEVICE_ROWS[view] ?? [])
    .map(key => schema.entries.find(e => e.key === key))
    .filter(
      (e): e is SettingEntry =>
        !!e && isShown(e, schema) && platformIncludes(e.platform, schema.device),
    )
}

export function useDeviceSettings(onError: ErrorSink) {
  /** The Global rows of the device settings, by key. */
  const rows = ref<Record<string, SettingRow>>({})
  /** The newest read: an older answer arriving later is dropped (engineering rule 7). */
  let latest = 0

  async function load() {
    const ask = ++latest
    try {
      const pages = await Promise.all(DEVICE_PAGES.map(p => emu.settings.page(p, GLOBAL)))
      if (ask !== latest) return
      rows.value = Object.fromEntries(
        pages
          .flat()
          .filter(r => DEVICE_KEYS.includes(r.key))
          .map(r => [r.key, r]),
      )
    } catch (e) {
      if (ask === latest) onError(e)
    }
  }

  async function set(key: SettingKey, v: SettingValue) {
    try {
      await emu.settings.set(key, GLOBAL, v)
    } catch (e) {
      onError(e)
    }
  }

  useSettingsChanged(DEVICE_KEYS, () => void load())

  return { rows, load, set }
}
