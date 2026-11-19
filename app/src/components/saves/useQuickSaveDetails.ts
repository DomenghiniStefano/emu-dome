// V03 opened from «…» on a slot tile: the quick save shown, and the focus back on that «…» when
// the dialog closes.
import { nextTick, ref } from 'vue'
import type { QuickSave } from '@/emu'

export function useQuickSaveDetails() {
  const quick = ref<QuickSave | null>(null)
  let opener: HTMLElement | null = null

  function open(q: QuickSave, e: Event) {
    opener = e.currentTarget instanceof HTMLElement ? e.currentTarget : null
    quick.value = q
  }
  function close() {
    quick.value = null
    void nextTick(() => opener?.focus())
  }

  return { quick, open, close }
}
