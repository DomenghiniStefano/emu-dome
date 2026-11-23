// The touch controls' surroundings, shared by the overlay and the C06 editor: the area they are
// laid out in (the window and its safe edges, following its size and rotations), each control's
// own box measured from the element (its design-system size, `data-touch-control`), and
// `touch.thide`'s fade.
import { nextTick, onBeforeUnmount, onMounted, ref, watch, type Ref } from 'vue'
import type { TouchControlId } from '@/emu'
import type { Area, Box, SafeEdges } from './touchLayout'

/** The window's safe edges (the camera's notch), read from the `safe-*` spacing tokens on an
 * invisible element: what the webview reports as `env(safe-area-inset-*)`. */
function readSafeEdges(probe: HTMLElement | null): SafeEdges {
  if (!probe) return { top: 0, right: 0, left: 0 }
  const style = getComputedStyle(probe)
  const px = (v: string) => {
    const n = parseFloat(v)
    return Number.isFinite(n) ? n : 0
  }
  return { top: px(style.paddingTop), right: px(style.paddingRight), left: px(style.paddingLeft) }
}

/** The window, in CSS pixels, and its safe edges, following resizes and rotations, and the safe
 * edges changing with no resize: Android hides its system bars once a game is open (#131) in a
 * window that keeps its size, so the probe's own box (its paddings) is watched too. */
export function useArea(): Ref<Area> {
  let probe: HTMLElement | null = null
  let observer: ResizeObserver | null = null
  const measure = (): Area => ({
    width: window.innerWidth,
    height: window.innerHeight,
    safe: readSafeEdges(probe),
  })
  const area = ref<Area>(measure())
  const resize = () => {
    const next = measure()
    const now = area.value
    const same =
      next.width === now.width &&
      next.height === now.height &&
      next.safe?.top === now.safe?.top &&
      next.safe?.right === now.safe?.right &&
      next.safe?.left === now.safe?.left
    if (!same) area.value = next
  }
  onMounted(() => {
    probe = document.createElement('div')
    probe.className = 'pointer-events-none invisible fixed pt-safe-top pr-safe-right pl-safe-left'
    probe.setAttribute('aria-hidden', 'true')
    document.body.appendChild(probe)
    window.addEventListener('resize', resize)
    if (typeof ResizeObserver !== 'undefined') {
      observer = new ResizeObserver(resize)
      observer.observe(probe, { box: 'border-box' })
    }
    resize()
  })
  onBeforeUnmount(() => {
    window.removeEventListener('resize', resize)
    observer?.disconnect()
    observer = null
    probe?.remove()
    probe = null
  })
  return area
}

/** Before a control is measured: the size of a face button. */
const UNMEASURED: Box = { w: 48, h: 48 }

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
