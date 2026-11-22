// How the library shows its games (#29): `library.view`, a per-device setting (the list on a
// phone, the grid elsewhere: the catalogue's default rule), read on mount and again when it
// changes elsewhere (Settings), written by the phone's Lista / Griglia switch. Null until read,
// so the view never shows one layout and then the other. A failure is reported and the value
// read back; with no value at all, `fallback` (the device's own default).
import { onMounted, ref } from 'vue'
import { useSettingsChanged } from '@/composables/useSettingsChanged'
import { emu } from '@/emu'
import type { ErrorSink } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import { GLOBAL, readSetting } from '@/lib/settingsModel'
import { isGamesView, type GamesView } from './libraryView'

const KEY = 'library.view'

export function useGamesView(fallback: () => GamesView, onError: ErrorSink) {
  const view = ref<GamesView | null>(null)
  // Rule 7: a read answered after a newer one started (or after a pick) is dropped.
  const reads = latest()

  async function load() {
    const fresh = reads.next()
    try {
      const row = await readSetting(KEY)
      if (fresh()) view.value = isGamesView(row?.value) ? row.value : fallback()
    } catch (e) {
      if (!fresh()) return
      view.value ??= fallback()
      onError(e)
    }
  }

  async function setView(v: GamesView) {
    reads.next()
    view.value = v
    try {
      await emu.settings.set(KEY, GLOBAL, v)
    } catch (e) {
      onError(e)
      await load()
    }
  }

  onMounted(() => void load())
  useSettingsChanged(KEY, () => void load())

  return { view, setView }
}
