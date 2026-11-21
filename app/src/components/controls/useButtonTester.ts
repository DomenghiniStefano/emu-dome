// «Prova i pulsanti» (C01, C02): what is held lights up, and the last press is described by the
// screen (`describe`). A controller is fed with `onPads` (every `emu://gamepad-state`); the keyboard
// is read here, from the window's keys, while `keyboard()` is true.
import { onBeforeUnmount, onMounted, ref } from 'vue'
import type { PadState } from '@/emu'
import { newlyHeld } from './tester'

export function useButtonTester(options: {
  /** The controller to read, or null for the keyboard. */
  padId: () => number | null
  describe: (source: string) => string
  /** While true (a capture running), presses light up but are not described. */
  paused?: () => boolean
}) {
  const held = ref(new Set<string>())
  /** The description of the last press; '' before the first. */
  const last = ref('')
  const paused = () => options.paused?.() ?? false

  function onPads(states: PadState[]) {
    const id = options.padId()
    if (id === null) return
    const now = new Set<string>(states.find(s => s.id === id)?.held ?? [])
    if (!paused())
      for (const source of newlyHeld(held.value, now)) last.value = options.describe(source)
    held.value = now
  }

  /** Back to nothing held and nothing said: another device was chosen. */
  function reset() {
    last.value = ''
    held.value = new Set()
  }

  function key(e: KeyboardEvent, down: boolean) {
    if (options.padId() !== null || paused()) return
    const next = new Set(held.value)
    if (down) {
      if (!next.has(e.code) && !e.repeat) last.value = options.describe(e.code)
      next.add(e.code)
    } else next.delete(e.code)
    held.value = next
  }
  const keydown = (e: KeyboardEvent) => key(e, true)
  const keyup = (e: KeyboardEvent) => key(e, false)
  onMounted(() => {
    window.addEventListener('keydown', keydown)
    window.addEventListener('keyup', keyup)
  })
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', keydown)
    window.removeEventListener('keyup', keyup)
  })

  return { held, last, onPads, reset }
}
