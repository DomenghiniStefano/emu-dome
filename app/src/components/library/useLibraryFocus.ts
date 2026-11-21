// The Library's first focus. The shell autofocuses on mount, before the library has loaded: then
// the focus sits on the rail's current item. Once the library is there, the view's own first
// control takes it (once); back from a game's details, that game's cover does.
import { nextTick, onMounted, watch, type Ref } from 'vue'
import type { GameId } from '@/emu'
import { library } from '@/lib/library'
import { focusInitial } from '@/nav'

/** The attribute a cover's click area carries, with its game's id. */
export const COVER_OF = 'data-cover-of'

/** `back`: the game whose details the view comes back from (the shell's route), read once. */
export function useLibraryFocus(root: Ref<HTMLElement | null>, back: GameId | null) {
  let focused = false
  async function focusOnce() {
    if (focused || !library.loaded) return
    focused = true
    await nextTick()
    const cover = back
      ? root.value?.querySelector<HTMLElement>(`[${COVER_OF}="${CSS.escape(back)}"]`)
      : null
    if (cover) {
      cover.focus({ preventScroll: true })
      cover.scrollIntoView({ block: 'nearest' })
      return
    }
    if (root.value) focusInitial(root.value, { unlessMoved: true })
  }
  watch(() => library.loaded, focusOnce)
  onMounted(() => void focusOnce())
}
