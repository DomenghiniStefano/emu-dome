// The settings the quick menu changes «in prova» (Session level): read for the open game, set
// live, listed, then saved for this game or its console (all or nothing) or undone. Closing the
// game drops them (the backend).
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSay } from '@/i18n/message'
import {
  emu,
  type SettingEntry,
  type SettingRow,
  type SettingsSchema,
  type SettingValue,
  type SettingKey,
} from '@/emu'
import { EVERY_SETTING, useSettingsChanged } from '@/composables/useSettingsChanged'
import type { ErrorSink } from '@/i18n/labels'
import { isShown, loadSchema } from '@/lib/settingsModel'
import { trialNames, trialTitle } from './trial'
import { SESSION_PAGES, SESSION_ROWS, type QuickMenuView } from './view'

const SESSION = { level: 'session' } as const

export function useTrialSession(onError: ErrorSink) {
  const { t } = useI18n()
  const say = useSay()
  const schema = ref<SettingsSchema | null>(null)
  /** The Session rows of the open game, by key. */
  const rows = ref<Record<string, SettingRow>>({})

  async function load() {
    try {
      const pages = await Promise.all(SESSION_PAGES.map(p => emu.settings.page(p, SESSION)))
      rows.value = Object.fromEntries(pages.flat().map(r => [r.key, r]))
    } catch (e) {
      onError(e)
    }
  }

  const entry = (key: SettingKey): SettingEntry | undefined =>
    schema.value?.entries.find(e => e.key === key)
  /** A setting's value for the open game, «in prova» included. */
  const value = (key: SettingKey): SettingValue | null =>
    rows.value[key]?.value ?? entry(key)?.default ?? null

  async function set(key: SettingKey, v: SettingValue) {
    try {
      await emu.settings.set(key, SESSION, v)
    } catch (e) {
      onError(e)
    }
  }
  async function commit(target: 'game' | 'console') {
    try {
      await emu.settings.commitSession(target)
    } catch (e) {
      onError(e)
    }
  }
  async function discard() {
    try {
      await emu.settings.discardSession()
    } catch (e) {
      onError(e)
    }
  }

  /** The changes «in prova», translated. */
  const names = computed(() => trialNames(Object.values(rows.value), entry).map(k => t(k)))
  /** The title over the changes; '' when there are none. */
  const title = computed(() => {
    if (names.value.length === 0) return ''
    return say(trialTitle(names.value))
  })

  /** The entries a page shows at the Session level. */
  function entriesOf(view: QuickMenuView): SettingEntry[] {
    const s = schema.value
    if (!s) return []
    return (SESSION_ROWS[view] ?? [])
      .map(key => entry(key))
      .filter((e): e is SettingEntry => !!e && isShown(e, s))
  }

  /** Reads the schema and the rows; every change from then on reads the rows again. */
  async function start() {
    schema.value = await loadSchema()
    await load()
  }
  useSettingsChanged(EVERY_SETTING, () => void load())

  return { schema, rows, value, set, commit, discard, names, title, entriesOf, start }
}
