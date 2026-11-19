// G10's controller shortcuts, read-only (they are edited in Controls, C03).
import { computed, ref } from 'vue'
import { emu, type ComboRow, type Pos } from '@/emu'
import type { ErrorSink } from '@/i18n/labels'

export function useControllerShortcuts(onError: ErrorSink) {
  const combos = ref<ComboRow[]>([])
  const modifier = ref<Pos>('l3')

  async function load() {
    try {
      const b = await emu.controls.bindings('generic', { level: 'session' })
      // Reserved shortcuts (features still to come) are listed in Controls (C03), not here.
      combos.value = b.combos.filter(c => c.status === 'active')
      modifier.value = b.modifier
    } catch (e) {
      onError(e)
    }
  }

  /** The shortcuts that have a button. */
  const boundCount = computed(() => combos.value.filter(r => r.pos !== null).length)

  return { combos, modifier, boundCount, load }
}
