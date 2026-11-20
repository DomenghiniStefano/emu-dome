// V01: what every library game has saved (`saves_summary`), the games that have saves, and the
// line the list shows for one («3 salvataggi rapidi · 1,3 MB»).
import { computed, ref, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type LibraryGame, type SavesSummary } from '@/emu'
import { sizeLabel, type ErrorSink } from '@/i18n/labels'
import { library } from '@/lib/library'
import { gamesWithSaves } from './savesModel'

export function useSavesOverview(slotCount: Ref<number>, fail: ErrorSink) {
  const { t } = useI18n()
  const summaries = ref<Record<string, SavesSummary>>({})
  /** The summaries of the loaded library are known. */
  const loaded = ref(false)

  // Rule 7: a load answered after a newer one started is dropped.
  let latest = 0

  async function load() {
    const request = ++latest
    try {
      const list = await emu.saves.summary(
        library.games.map(g => g.id),
        slotCount.value,
      )
      if (request !== latest) return
      summaries.value = Object.fromEntries(list.map(s => [s.game, s]))
    } catch (e) {
      if (request !== latest) return
      fail(e)
    }
    loaded.value = library.loaded
  }

  const withSaves = computed(() => gamesWithSaves(library.games, summaries.value))

  function detailOf(g: LibraryGame): string {
    const s = summaries.value[g.id]
    if (!s) return ''
    const size = sizeLabel(s.bytes)
    return s.quickCount > 0
      ? t('saves.metaQuick', { n: s.quickCount, size }, s.quickCount)
      : t('saves.metaInGame', { size })
  }

  const isBroken = (g: LibraryGame) => !!summaries.value[g.id]?.broken

  return { loaded, withSaves, load, detailOf, isBroken }
}
