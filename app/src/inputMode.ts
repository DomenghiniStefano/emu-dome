// The input in use, so hints show only its keys (quick menu, 18/11): keyboard keys with mouse
// and keyboard, controller glyphs with a controller, nothing on touch. The last input wins.
import { ref } from 'vue'

export type InputMode = 'keyboard' | 'controller' | 'touch'

export const inputMode = ref<InputMode>(
  window.matchMedia('(pointer: coarse)').matches ? 'touch' : 'keyboard',
)

/** A step or the menu came from a controller (`emu://nav`, `emu://menu`). */
export function usedController() {
  inputMode.value = 'controller'
}

let watching = false
/** Keys and the mouse mean keyboard; a finger means touch. Call once at startup. */
export function watchInput() {
  if (watching) return
  watching = true
  window.addEventListener('keydown', () => (inputMode.value = 'keyboard'), { capture: true })
  window.addEventListener(
    'pointerdown',
    e => (inputMode.value = e.pointerType === 'touch' ? 'touch' : 'keyboard'),
    { capture: true },
  )
}
