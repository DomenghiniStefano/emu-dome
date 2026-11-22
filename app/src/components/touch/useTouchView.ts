// The touch controls' surroundings, shared by the overlay and the C06 editor: the area they are
// laid out in (the window, following its size and rotations), each control's own box measured
// from the element (its design-system size, `data-touch-control`), and `touch.thide`'s fade.
import { nextTick, onBeforeUnmount, onMounted, ref, watch, type Ref } from 'vue'
import type { TouchControlId } from '@/emu'
import type { Area, Box } from './touchLayout'

/** The window, in CSS pixels, following resizes and rotations. */
export function useArea(): Ref<Area> {
  const area = ref<Area>({ width: window.innerWidth, height: window.innerHeight })
  const resize = () => (area.value = { width: window.innerWidth, height: window.innerHeight })
  onMounted(() => window.addEventListener('resize', resize))
  onBeforeUnmount(() => window.removeEventListener('resize', resize))
  return area
}

/** Before a control is measured: the size of a face button. */
const UNMEASURED: Box = { w: 64, h: 64 }

/**
 * Each control's own box (unscaled: `offsetWidth` ignores the transform), read from the controls
 * under `root` after every render of `controls`.
 */
export function useControlBoxes(root: Ref<HTMLElement | null>, controls: () => unknown) {
  const boxes = ref(new Map<TouchControlId, Box>())
  function measure() {
    const next = new Map<TouchControlId, Box>()
    root.value?.querySelectorAll<HTMLElement>('[data-touch-control]').forEach(el => {
      const id = el.dataset.touchControl as TouchControlId
      if (el.offsetWidth > 0) next.set(id, { w: el.offsetWidth, h: el.offsetHeight })
    })
    const same =
      next.size === boxes.value.size &&
      [...next].every(([id, b]) => {
        const old = boxes.value.get(id)
        return old?.w === b.w && old.h === b.h
      })
    if (!same) boxes.value = next
  }
  watch(controls, () => void nextTick(measure), { flush: 'post', deep: true })
  onMounted(() => void nextTick(measure))
  const boxOf = (id: TouchControlId): Box => boxes.value.get(id) ?? UNMEASURED
  return { boxes, boxOf, measure }
}

/**
 * `touch.thide`: true once `after()` ms passed without a touch anywhere (null: never); the next
 * touch brings the controls back and starts the count again.
 */
export function useIdleFade(after: () => number | null) {
  const faded = ref(false)
  let timer: ReturnType<typeof setTimeout> | undefined
  function restart() {
    clearTimeout(timer)
    faded.value = false
    const ms = after()
    if (ms !== null) timer = setTimeout(() => (faded.value = true), ms)
  }
  watch(after, restart)
  onMounted(() => {
    window.addEventListener('pointerdown', restart, { capture: true })
    restart()
  })
  onBeforeUnmount(() => {
    window.removeEventListener('pointerdown', restart, { capture: true })
    clearTimeout(timer)
  })
  return faded
}
