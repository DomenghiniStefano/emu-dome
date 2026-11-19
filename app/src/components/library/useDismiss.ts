// Closing a popover held next to its toggle the way the library's do (L03 filters, the «…» menu
// of a game) with a press outside the area that holds both. The toggle toggles the popover itself,
// so a press on it is not «outside» (closing here too would reopen it on the same click). B from
// the controller and Esc are the popover's nav scope `back` (nav.ts, `escape`).
import { onBeforeUnmount, onMounted, type Ref } from 'vue'

/** Whether a press on `target` falls outside `area`. No area yet: not outside. */
export function pressedOutside(area: Element | null | undefined, target: EventTarget | null) {
  return !!area && target instanceof Node && !area.contains(target)
}

/** Closes the popover on a pointer press outside its parent element (popover and toggle). */
export function useDismiss(root: Ref<HTMLElement | null>, close: () => void) {
  function onPointer(e: PointerEvent) {
    if (pressedOutside(root.value?.parentElement, e.target)) close()
  }
  onMounted(() => window.addEventListener('pointerdown', onPointer, { capture: true }))
  onBeforeUnmount(() => window.removeEventListener('pointerdown', onPointer, { capture: true }))
}
