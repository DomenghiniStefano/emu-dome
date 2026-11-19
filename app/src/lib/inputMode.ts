// The input in use, so hints show only its keys (quick menu, 18/11): keyboard keys with mouse
// and keyboard, controller glyphs with a controller, nothing on touch. The last input wins.
import { ref } from 'vue'
import { coarsePointer } from '@/composables/useDeviceLayout'

export type InputMode = 'keyboard' | 'controller' | 'touch'

export const inputMode = ref<InputMode>(coarsePointer() ? 'touch' : 'keyboard')

/** A step or the menu came from a controller (`emu://nav`, `emu://menu`). */
export function usedController() {
  inputMode.value = 'controller'
}

let watching = false
/** Keys and the mouse mean keyboard; a finger means touch. Call once at startup. */
export function watchInput() {
  if (watching) return
  watching = true
  // Only a real key press: nav.ts sends a controller step to a design-system primitive as a key
  // event of its own (untrusted), and the controller must stay the input in use.
  window.addEventListener('keydown', e => e.isTrusted && (inputMode.value = 'keyboard'), {
    capture: true,
  })
  window.addEventListener(
    'pointerdown',
    e => (inputMode.value = e.pointerType === 'touch' ? 'touch' : 'keyboard'),
    { capture: true },
  )
}
