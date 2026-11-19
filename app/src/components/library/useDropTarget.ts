// Files or folders dragged over the window, outside the game: the drop target shows, and what is
// dropped goes into the library.
import { ref, type Ref } from 'vue'
import { emu, type GameInfo } from '@/emu'
import type { Toasts } from '@/composables/useToasts'

export function useDropTarget(game: Ref<GameInfo | null>, toasts: Toasts) {
  const dragOver = ref(false)

  async function drop(paths: string[]) {
    dragOver.value = false
    if (game.value || paths.length === 0) return
    try {
      await emu.library.addPaths(paths)
    } catch (e) {
      toasts.failWith(e)
    }
  }

  /** Follows the window's drag and drop events. */
  const listen = () =>
    emu.library.onDragDrop(e => {
      if (e.type === 'over') dragOver.value = !game.value
      else if (e.type === 'leave') dragOver.value = false
      else void drop(e.paths)
    })

  return { dragOver, listen }
}
