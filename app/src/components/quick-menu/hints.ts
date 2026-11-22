// Hints show only the input in use (quick menu, 18/11): Esc with mouse and keyboard, B with a
// controller, none on touch; the same for the search's keys.
import type { InputMode } from '@/lib/inputMode'

/** The message key of the key that goes back or resumes, or null on touch. */
export function backKeyLabel(mode: InputMode): string | null {
  switch (mode) {
    case 'keyboard':
      return 'flow.hintEsc'
    case 'controller':
      return 'controls.button.b'
    case 'touch':
      return null
  }
}

/** The message keys of the keys that open the search (X01, #32): Ctrl K, Y, none on touch. */
export function searchKeyLabels(mode: InputMode): string[] {
  switch (mode) {
    case 'keyboard':
      return ['search.key.ctrl', 'search.key.k']
    case 'controller':
      return ['controls.button.y']
    case 'touch':
      return []
  }
}
