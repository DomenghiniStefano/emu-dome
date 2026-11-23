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

/**
 * A physical keyboard has been used in this run (Trello #118): kept until the app quits. On the
 * phone the keyboard's keys (C01's «Tasti della tastiera», C03's keyboard tab, G06's Esc, the
 * search's F11) show only once this is true.
 */
export const keyboardSeen = ref(false)

/** Media, volume and system keys a phone or a remote sends without being a keyboard. */
const NOT_A_KEYBOARD = /^(Audio|Media|Browser|Launch|Power|Sleep|WakeUp|Lang|Unidentified)/

/**
 * Whether `e` came from a physical keyboard: a trusted press with a key code. A virtual
 * keyboard (Android's IME) sends no code (`""`) or `Unidentified` while it composes; nav.ts's
 * own key events for a controller step are untrusted.
 */
export function fromPhysicalKeyboard(
  e: Pick<KeyboardEvent, 'isTrusted' | 'code' | 'key' | 'isComposing'>,
): boolean {
  return (
    e.isTrusted &&
    !e.isComposing &&
    e.code !== '' &&
    !NOT_A_KEYBOARD.test(e.code) &&
    e.key !== 'Unidentified' &&
    e.key !== 'Process'
  )
}

/**
 * Whether the keyboard's keys are worth showing: always on a computer; on the phone (the touch
 * layout, upright or on its side) only once a physical keyboard has been used. The same rule as
 * the quick menu's hints: only the input that is there.
 */
export function showKeyboardKeys(phone: boolean, seen: boolean): boolean {
  return !phone || seen
}

let watching = false
/** Keys and the mouse mean keyboard; a finger means touch. Call once at startup. */
export function watchInput() {
  if (watching) return
  watching = true
  // Only a real key press: nav.ts sends a controller step to a design-system primitive as a key
  // event of its own (untrusted), and the controller must stay the input in use.
  window.addEventListener(
    'keydown',
    e => {
      if (e.isTrusted) inputMode.value = 'keyboard'
      if (!keyboardSeen.value && fromPhysicalKeyboard(e)) keyboardSeen.value = true
    },
    { capture: true },
  )
  window.addEventListener(
    'pointerdown',
    e => (inputMode.value = e.pointerType === 'touch' ? 'touch' : 'keyboard'),
    { capture: true },
  )
}
