// First run (#35): whether this device still needs it. Null until Rust answered.
import { ref } from 'vue'
import { emu } from '@/emu'
import type { Toasts } from '@/composables/useToasts'

export function useFirstRun(toasts: Toasts) {
  const firstRun = ref<boolean | null>(null)
  async function decide() {
    try {
      firstRun.value = await emu.firstRun.show()
    } catch (e) {
      firstRun.value = false
      toasts.failWith(e)
    }
  }
  const finish = () => (firstRun.value = false)
  return { firstRun, decide, finish }
}
