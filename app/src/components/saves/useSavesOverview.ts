// V01: what every library game has saved (`saves_summary`), the games that have saves, and the
// line the list shows for one («3 salvataggi rapidi · 1,3 MB»).
import { computed, ref, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type LibraryGame, type SavesSummary } from '../../emu'
import { sizeLabel } from '../../i18n/labels'
import { library } from '../../library'
import { gamesWithSaves } from './savesModel'

export function useSavesOverview(slotCount: Ref<number>, fail: (e: unknown) => void) {
  const { t } = useI18n()
  const summaries = ref<Record<string, SavesSummary>>({})
  /** The summaries of the loaded library are known. */
  const loaded = ref(false)

  async function load() {
    try {
      const list = await emu.saves.summary(
        library.games.map(g => g.path),
        slotCount.value,
      )
      summaries.value = Object.fromEntries(list.map(s => [s.path, s]))
    } catch (e) {
      fail(e)
    }
    loaded.value = library.loaded
  }

  const withSaves = computed(() => gamesWithSaves(library.games, summaries.value))

  function detailOf(g: LibraryGame): string {
    const s = summaries.value[g.path]
    if (!s) return ''
    const size = sizeLabel(s.bytes)
    return s.quickCount > 0
      ? t('saves.metaQuick', { n: s.quickCount, size }, s.quickCount)
      : t('saves.metaInGame', { size })
  }

  const isBroken = (g: LibraryGame) => !!summaries.value[g.path]?.broken

  return { loaded, withSaves, load, detailOf, isBroken }
}
