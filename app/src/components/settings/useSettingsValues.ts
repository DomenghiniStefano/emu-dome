// The values the Settings section shows and writes: every page at the level being edited, the
// recent changes of T01, and the writes (set, reset, the background image picker). Reloaded when
// the level changes and when a setting changes anywhere (`settings://changed`).
import { reactive, ref, watch, type Ref } from 'vue'
import {
  emu,
  type SettingsChanged,
  type SettingEntry,
  type SettingRecent,
  type SettingsSchema,
  type SettingsScope,
  type SettingValue,
  type SettingKey,
} from '@/emu'
import type { ErrorSink } from '@/i18n/labels'
import { latest, latestBy, type Fresh } from '@/lib/latest'
import { isShown } from '@/lib/settingsModel'
import type { PageValues } from './settingsPage'

/** How many recent changes T01 lists. */
const RECENT = 5

export function useSettingsValues(
  schema: Ref<SettingsSchema | null>,
  byKey: Ref<Map<string, SettingEntry>>,
  scope: Ref<SettingsScope>,
  onError: ErrorSink,
) {
  /** Values of every page at the current scope, by page then key. */
  const values = reactive<Record<string, PageValues>>({})
  const recent = ref<SettingRecent[]>([])

  /** Runs a bridge call, reporting a failure as an error toast unless it is no longer `fresh`. */
  async function attempt(call: () => Promise<unknown>, fresh: Fresh = () => true) {
    try {
      await call()
    } catch (e) {
      if (fresh()) onError(e)
    }
  }

  // Rule 7: quick L1 / R1 change the scope while its pages load; an answer for an older scope,
  // or an older load of the same page, is dropped.
  const scopeLoads = latest()
  const pageLoads = latestBy<string>()
  const recentLoads = latest()

  function loadPage(page: string) {
    const ofScope = scopeLoads.now()
    const ofPage = pageLoads.next(page)
    const fresh = () => ofScope() && ofPage()
    return attempt(async () => {
      const rows = await emu.settings.page(page, scope.value)
      if (fresh()) values[page] = Object.fromEntries(rows.map(r => [r.key, r]))
    }, fresh)
  }
  async function loadAll() {
    if (!schema.value) return
    scopeLoads.next()
    await Promise.all(schema.value.pages.map(loadPage))
  }
  watch(scope, () => void loadAll(), { deep: true })

  function loadRecent() {
    const fresh = recentLoads.next()
    return attempt(async () => {
      const s = schema.value
      const rows = await emu.settings.recent(RECENT)
      if (!fresh()) return
      recent.value = rows.filter(r => {
        const e = byKey.value.get(r.key)
        return !!e && !!s && isShown(e, s)
      })
    }, fresh)
  }

  const set = (key: SettingKey, value: SettingValue) =>
    attempt(() => emu.settings.set(key, scope.value, value))
  const reset = (key: SettingKey) => attempt(() => emu.settings.reset(key, scope.value))
  /** «Scegli un’immagine…» (`video.surroundImg`): the picker, then the value at this scope. */
  const chooseBackground = () => attempt(() => emu.settings.chooseBackground(scope.value))

  /** A change made anywhere: its page and the recent changes are read again. */
  function onChanged(e: SettingsChanged) {
    // The page from the schema: a key's prefix is not always its page («Avanzate»).
    const p = byKey.value.get(e.key)?.page
    if (p) void loadPage(p)
    void loadRecent()
  }

  return { values, recent, loadAll, loadRecent, set, reset, chooseBackground, onChanged }
}
