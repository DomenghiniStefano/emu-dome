// The room the phone's shell leaves to its sections (#142): between the top bar and the tab bar
// upright, beside the side column on its side. The toasts are drawn by the root (App.vue, over
// the dialogs too), which cannot see the shell's bars: AppShell measures the room its sections
// get and reports it here, the root reads it and places the stack inside it. The desktop and TV
// report nothing (their stack keeps its own place, past the rail).
import { inject, onScopeDispose, provide, ref, watch, type InjectionKey, type Ref } from 'vue'

/** The sections' room, as the distance of each of its edges from the window's, in CSS px. */
export interface ShellArea {
  left: number
  right: number
  bottom: number
}

const KEY: InjectionKey<Ref<ShellArea | null>> = Symbol('shellArea')

/** The root: the room the shell reports, null when it reports none (no shell, the desktop). */
export function provideShellArea(): Ref<ShellArea | null> {
  const area = ref<ShellArea | null>(null)
  provide(KEY, area)
  return area
}

/** The room `el` takes in a window of `width` × `height`. */
export function areaOf(rect: DOMRect, width: number, height: number): ShellArea {
  return {
    left: Math.round(rect.left),
    right: Math.round(width - rect.right),
    bottom: Math.round(height - rect.bottom),
  }
}

/**
 * The shell: reports the room `el` takes while `active` (the phone's layouts), following its
 * size and the window's (a turn of the phone moves the bars); nothing otherwise. A turn from one
 * side to the other (Android's navigation bar from the right to the left) changes neither size,
 * only where the room is: the screen's orientation change measures it again, once the layout
 * has followed (#142).
 */
export function useShellAreaReport(el: Ref<HTMLElement | null>, active: () => boolean) {
  const area = inject(KEY, null)
  if (!area) return
  const measure = () => {
    const node = el.value
    area.value =
      node && active()
        ? areaOf(node.getBoundingClientRect(), window.innerWidth, window.innerHeight)
        : null
  }
  /** The insets move after the orientation event: measured now and on the next two frames. */
  let frame = 0
  const turned = () => {
    measure()
    window.cancelAnimationFrame(frame)
    frame = window.requestAnimationFrame(() => {
      measure()
      frame = window.requestAnimationFrame(measure)
    })
  }
  const orientation = typeof screen !== 'undefined' ? screen.orientation : undefined
  orientation?.addEventListener('change', turned)
  const observer = typeof ResizeObserver !== 'undefined' ? new ResizeObserver(measure) : null
  watch(
    [el, active],
    ([node], [old]) => {
      if (old) observer?.unobserve(old)
      if (node) observer?.observe(node)
      measure()
    },
    { flush: 'post' },
  )
  window.addEventListener('resize', measure)
  onScopeDispose(() => {
    observer?.disconnect()
    window.removeEventListener('resize', measure)
    orientation?.removeEventListener('change', turned)
    window.cancelAnimationFrame(frame)
    area.value = null
  })
}
