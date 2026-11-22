// «Cosa sincronizzare» (Trello #31): the `cloud` page of the settings schema at «Tutti i giochi»,
// its sections and rows as the Settings section shows them (settingsPage.ts), without the
// account's section (its rows are the Cloud section's head and status card). Nothing listed by
// hand: a new `cloud.*` entry shows here by itself. Read again on a change made anywhere.
import { computed, onMounted, ref } from 'vue'
import { emu, type SettingKey, type SettingsSchema, type SettingValue } from '@/emu'
import { useSettingsChanged } from '@/composables/useSettingsChanged'
import type { ErrorSink } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import { GLOBAL, loadSchema } from '@/lib/settingsModel'
import { pageSections, type PageValues } from '@/components/settings/settingsPage'

/** The settings page of the sync. */
export const CLOUD_PAGE = 'cloud'
/** Its section shown elsewhere: the account and the Drive space. */
const ACCOUNT_SECTION = 'account'

export function useCloudSettings(onError: ErrorSink) {
  const schema = ref<SettingsSchema | null>(null)
  const rows = ref<PageValues>({})
  const byKey = computed(() => new Map((schema.value?.entries ?? []).map(e => [e.key, e])))
  const loads = latest()

  async function load() {
    const fresh = loads.next()
    try {
      const s = await loadSchema()
      const page = await emu.settings.page(CLOUD_PAGE, GLOBAL)
      if (!fresh()) return
      schema.value = s
      rows.value = Object.fromEntries(page.map(r => [r.key, r]))
    } catch (e) {
      if (fresh()) onError(e)
    }
  }
  onMounted(load)
  useSettingsChanged(
    e => byKey.value.get(e.key)?.page === CLOUD_PAGE,
    () => void load(),
  )

  const sections = computed(() => {
    const s = schema.value
    if (!s) return []
    return pageSections(s, CLOUD_PAGE, { [CLOUD_PAGE]: rows.value }, byKey.value, null).filter(
      sec => sec.id !== ACCOUNT_SECTION,
    )
  })

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

  return { schema, rows, sections, set, reset }
}
