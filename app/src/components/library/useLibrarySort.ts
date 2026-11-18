// The library's order: the `library.sort` setting at the Global level, read on mount, written
// when the user picks one in L03, reset with the panel's «Ripristina», and read again when the
// setting changes elsewhere (Settings). A failure is reported as text and the order read back.
import { onMounted, onUnmounted, ref } from 'vue'
import { emu, type SettingsScope } from '../../emu'
import { errorText } from '../../i18n/labels'
import { DEFAULT_SORT, isSort, type Sort } from '../../librarySort'

const GLOBAL: SettingsScope = { level: 'global' }
const KEY = 'library.sort'

export function useLibrarySort(onError: (text: string) => void) {
  const sort = ref<Sort>(DEFAULT_SORT)

  async function load() {
    try {
      const row = (await emu.settings.page('library', GLOBAL)).find(r => r.key === KEY)
      sort.value = isSort(row?.value) ? row.value : DEFAULT_SORT
    } catch (e) {
      onError(errorText(e))
    }
  }

  async function setSort(s: Sort) {
    sort.value = s
    try {
      await emu.settings.set(KEY, GLOBAL, s)
    } catch (e) {
      onError(errorText(e))
      await load()
    }
  }

  async function resetSort() {
    try {
      await emu.settings.reset(KEY, GLOBAL)
    } catch (e) {
      onError(errorText(e))
    }
    await load()
  }

  let unlisten: (() => void) | null = null
  onMounted(async () => {
    await load()
    unlisten = await emu.settings.onChanged(c => {
      if (c.key === KEY) void load()
    })
  })
  onUnmounted(() => unlisten?.())

  return { sort, setSort, resetSort }
}
