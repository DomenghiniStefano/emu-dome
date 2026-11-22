// One search everywhere for the whole app (X01, #32): App.vue mounts SearchPalette once, and the
// screen it opens over registers itself as the search's host while it is mounted: the shell (no
// game) or the quick menu (a game open). The host says where the search is (`context`, what can
// be chosen there) and what choosing a result does; the newest host mounted is the one in use.
//
// Opening: the host's `openSearch` (the shell's top bar field, Y through its nav scope's `search`
// step), or Ctrl + K (⌘ K) while the host's scope has the steps (a dialog or a menu open above it
// keeps them). A host that goes (the game closes under the quick menu) closes the search with it.
import { computed, onBeforeUnmount, onMounted, reactive, shallowReactive, type Ref } from 'vue'
import type { LibraryGame, SettingKey } from '@/emu'
import type { KeyAction } from '@/lib/keyboard'
import { activeScopeRoot } from '@/nav'
import type { SearchContext } from './searchContext'

export interface SearchHost {
  /** The host's nav scope: Ctrl + K opens the search only while it has the steps. */
  root: Ref<HTMLElement | null>
  /** Where the search is now (read reactively). */
  context: () => SearchContext
  /** An action without a reason (searchContext.ts) was chosen. */
  action: (id: KeyAction) => void
  /** A setting without a reason was chosen. */
  setting: (key: SettingKey) => void
  /** A game without a reason was chosen; `start`: Ctrl + Enter. */
  game: (game: LibraryGame, start: boolean) => void
}

const hosts = shallowReactive<SearchHost[]>([])

/** Whether the palette is open. */
export const searchState = reactive({ open: false })

/** The host in use: the newest mounted. */
export const searchHost = computed<SearchHost | null>(() => hosts.at(-1) ?? null)

/** Registers `host` while the calling component is mounted. */
export function useSearchHost(host: SearchHost) {
  function openSearch() {
    if (searchHost.value === host) searchState.open = true
  }

  function onKey(e: KeyboardEvent) {
    if (e.code !== 'KeyK' || !(e.ctrlKey || e.metaKey) || e.altKey || e.shiftKey) return
    if (!host.root.value || activeScopeRoot() !== host.root.value) return
    e.preventDefault()
    openSearch()
  }

  onMounted(() => {
    hosts.push(host)
    window.addEventListener('keydown', onKey)
  })
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKey)
    const i = hosts.indexOf(host)
    if (i < 0) return
    const current = i === hosts.length - 1
    hosts.splice(i, 1)
    if (current) searchState.open = false
  })

  return { openSearch }
}
