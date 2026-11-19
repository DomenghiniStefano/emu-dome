// The first run's steps (#35): the list (S04 only while a controller is connected, from Rust),
// the current one, and moving between them.
import { computed, ref, type Ref } from 'vue'
import { emu, type FirstRunStep, type PadInfo } from '../emu'
import type { ErrorSink } from '../i18n/labels'
import { DEFAULT_STEPS, mainActionKey, stepAfterChange } from '../lib/first-run/steps'

export function useFirstRunSteps(pads: Ref<PadInfo[]>, on: { error: ErrorSink; done: () => void }) {
  const steps = ref<FirstRunStep[]>([...DEFAULT_STEPS])
  const step = ref<FirstRunStep>('welcome')
  const index = computed(() => Math.max(0, steps.value.indexOf(step.value)))
  const isLast = computed(() => index.value === steps.value.length - 1)
  /** The main button's message key: «Inizia», «Continua», «Fine». */
  const mainKey = computed(() => mainActionKey(index.value, steps.value.length))

  /** The steps for the controllers connected now; a step that went away moves to the one now
   * in its place (the controller unplugged on S04 → S06). */
  async function refresh() {
    try {
      const next = await emu.firstRun.steps(pads.value.length > 0)
      step.value = stepAfterChange(step.value, index.value, next)
      steps.value = next
    } catch (e) {
      on.error(e)
    }
  }

  async function finish() {
    try {
      await emu.firstRun.done()
    } catch (e) {
      on.error(e)
    }
    on.done()
  }

  function next() {
    if (isLast.value) return void finish()
    step.value = steps.value[index.value + 1] ?? step.value
  }
  function back() {
    if (index.value > 0) step.value = steps.value[index.value - 1] ?? step.value
  }

  return { steps, step, index, mainKey, refresh, next, back }
}
