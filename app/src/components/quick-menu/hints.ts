// Hints show only the input in use (quick menu, 18/11): Esc with mouse and keyboard, B with a
// controller, none on touch.
import type { InputMode } from '@/inputMode'

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
