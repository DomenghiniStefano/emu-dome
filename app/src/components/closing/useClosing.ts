// The close flow in the interface (Trello #86): follows `emu://closing` and sends the user's
// choice back. The flow runs in Rust; App.vue listens once and ClosingOverlay.vue draws `view`.
import { computed, ref } from 'vue'
import { emu, type ClosingProgress } from '@/emu'
import { flow } from '@/components/game/gameFlow'
import { useListeners } from '@/composables/useListeners'
import type { Toasts } from '@/composables/useToasts'
import { closingView } from './closing'

export function useClosing(toasts: Toasts) {
  const progress = ref<ClosingProgress | null>(null)
  const view = computed(() => closingView(progress.value))
  const listeners = useListeners()

  async function listen() {
    await listeners.add(
      emu.onClosing(p => {
        progress.value = p
        // The core's `stopped` is expected now, not E11.
        flow.ending = p.phase !== 'closed'
      }),
    )
  }

  /** «Wait»: keep waiting for the saves. */
  async function wait() {
    try {
      await emu.closeWait()
    } catch (e) {
      toasts.failWith(e)
    }
  }

  /** «Close anyway»: close without the saves still missing. */
  async function closeAnyway() {
    try {
      await emu.closeAnyway()
    } catch (e) {
      toasts.failWith(e)
    }
  }

  return { view, listen, wait, closeAnyway }
}
