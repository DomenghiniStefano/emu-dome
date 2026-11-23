// Whether a screen shows the keyboard's keys (Trello #118): on a computer always, on the phone
// (the touch layout, upright or on its side) only once a physical keyboard has been used in this
// run (lib/inputMode.ts, `showKeyboardKeys`). Used by Controls (C01, C03, E05), G06's hint and
// the search's shortcuts.
import { computed, type ComputedRef } from 'vue'
import { keyboardSeen, showKeyboardKeys } from '@/lib/inputMode'
import { useDeviceLayout } from './useDeviceLayout'

export function useKeyboardKeys(): ComputedRef<boolean> {
  const { touchDensity } = useDeviceLayout()
  return computed(() => showKeyboardKeys(touchDensity.value, keyboardSeen.value))
}
