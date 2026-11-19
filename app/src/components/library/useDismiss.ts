// Closing a popover the way the library's do (L03 filters, the «…» menu of a game, L12 cover
// picker): Esc from the keyboard, and, for a popover held next to its toggle, a press outside the
// area that holds both. The toggle toggles the popover itself, so a press on it is not «outside»
// (closing here too would reopen it on the same click). B from the controller is the nav scope's
// `back`, registered by the popover.
import { onBeforeUnmount, onMounted, type Ref } from 'vue'

export interface DismissOptions {
  /** Close on a pointer press outside the popover's parent element (popover and toggle). */
  outside?: boolean
}

/** Whether a press on `target` falls outside `area`. No area yet: not outside. */
export function pressedOutside(area: Element | null | undefined, target: EventTarget | null) {
  return !!area && target instanceof Node && !area.contains(target)
}

export function useDismiss(
  root: Ref<HTMLElement | null>,
  close: () => void,
  { outside = false }: DismissOptions = {},
) {
  function onKey(e: KeyboardEvent) {
    if (e.code === 'Escape') close()
  }
  function onPointer(e: PointerEvent) {
    if (pressedOutside(root.value?.parentElement, e.target)) close()
  }
  onMounted(() => {
    window.addEventListener('keydown', onKey)
    if (outside) window.addEventListener('pointerdown', onPointer, { capture: true })
  })
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKey)
    if (outside) window.removeEventListener('pointerdown', onPointer, { capture: true })
  })
}
