// The library's order: the `library.sort` setting at the Global level, read on mount, written
// when the user picks one in L03, reset with the panel's «Ripristina», and read again when the
// setting changes elsewhere (Settings). A failure is reported and the order read back.
import { onMounted, ref } from 'vue'
import { useSettingsChanged } from '@/composables/useSettingsChanged'
import { emu } from '@/emu'
import type { ErrorSink } from '@/i18n/labels'
import { DEFAULT_SORT, isSort, type Sort } from './librarySort'
import { GLOBAL, readSetting } from '@/lib/settingsModel'

const KEY = 'library.sort'

export function useLibrarySort(onError: ErrorSink) {
  const sort = ref<Sort>(DEFAULT_SORT)

  async function load() {
    try {
      const row = await readSetting(KEY)
      sort.value = isSort(row?.value) ? row.value : DEFAULT_SORT
    } catch (e) {
      onError(e)
    }
  }

  async function setSort(s: Sort) {
    sort.value = s
    try {
      await emu.settings.set(KEY, GLOBAL, s)
    } catch (e) {
      onError(e)
      await load()
    }
  }

  async function resetSort() {
    try {
      await emu.settings.reset(KEY, GLOBAL)
    } catch (e) {
      onError(e)
    }
    await load()
  }

  onMounted(() => void load())
  useSettingsChanged(KEY, () => void load())

  return { sort, setSort, resetSort }
}
