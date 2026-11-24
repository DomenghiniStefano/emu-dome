// Which inputs' keys a text may name now (#148): the keyboard's and the controller's, by the rule
// of #118 (lib/inputMode.ts `keysShown`), following the device, a keyboard used and the
// controllers connected. Used by a setting row whose description names keys.
import { computed, type ComputedRef } from 'vue'
import { keyboardSeen, keysShown } from '@/lib/inputMode'
import { uiMode } from './useUiMode'
import { useDeviceLayout } from './useDeviceLayout'

export function useKeysShown(): ComputedRef<{ keyboard: boolean; controller: boolean }> {
  const { touchDensity } = useDeviceLayout()
  return computed(() => keysShown(touchDensity.value, keyboardSeen.value, uiMode.pad !== null))
}
