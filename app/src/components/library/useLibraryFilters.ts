// What the grid shows: L03's «Mostra» switches and the console chip over the library, in the
// chosen order. A chip whose console left the list (filters, removal) falls back to «Tutti».
import { computed, reactive, ref, watch, type Ref } from 'vue'
import type { ConsoleId } from '@/emu'
import { library } from '@/lib/library'
import { SHOW_KEYS, shown, sorted, type ShowKey, type Sort } from './librarySort'
import { consoleChips, ofConsole } from './libraryView'

export function useLibraryFilters(sort: Ref<Sort>) {
  const show = reactive<Record<ShowKey, boolean>>({
    favourites: false,
    unplayed: false,
    here: false,
    hidden: false,
  })
  const consoleFilter = ref<ConsoleId | 'all'>('all')

  const base = computed(() => shown(library.games, show))
  const chips = computed(() => consoleChips(base.value))
  watch(chips, list => {
    if (!list.some(c => c.id === consoleFilter.value)) consoleFilter.value = 'all'
  })
  const games = computed(() => sorted(ofConsole(base.value, consoleFilter.value), sort.value))

  /** Any switch or chip narrowing the grid: «Togli i filtri» is offered. */
  const filtering = computed(() => consoleFilter.value !== 'all' || SHOW_KEYS.some(k => show[k]))

  function toggleShow(k: ShowKey) {
    show[k] = !show[k]
  }
  function clearShow() {
    for (const k of SHOW_KEYS) show[k] = false
  }
  /** «Togli i filtri» under «Nessun gioco corrisponde…»: the switches and the console chip. */
  function clearFilters() {
    clearShow()
    consoleFilter.value = 'all'
  }

  return { show, consoleFilter, chips, games, filtering, toggleShow, clearShow, clearFilters }
}
