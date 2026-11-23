// When a tooltip shows (Trello #104): the name of an icon alone, for the mouse after a short rest
// over it, and for the controller and the keyboard as soon as the focus lands on it (the focus
// ring is showing, `data-nav` on <html>, or the browser says `:focus-visible`). A press hides it
// and the focus that follows a press does not bring it back; a touch never shows it. Esc hides it
// first, in the capture phase, so the dialog or the screen behind still gets that same Esc: Reka
// closes only its highest layer, and the open tooltip would be that layer.
import { onBeforeUnmount, ref, shallowRef, watch } from 'vue'

/** How long the mouse rests on the element before its tooltip shows, in ms. */
export const TOOLTIP_DELAY_MS = 400

/** Whether a focus that landed on `el` came from the controller or the keyboard. */
export function focusShowsTooltip(el: Element): boolean {
  if ('nav' in document.documentElement.dataset) return true
  try {
    return el.matches(':focus-visible')
  } catch {
    // A DOM without the selector (the unit tests' jsdom): only the focus ring says it.
    return false
  }
}

/** The open state of one element's tooltip and the listeners that drive it. */
export function useTooltip() {
  const open = ref(false)
  const anchor = shallowRef<HTMLElement | null>(null)
  let timer: ReturnType<typeof setTimeout> | undefined
  let pressed = false

  function clear() {
    if (timer === undefined) return
    clearTimeout(timer)
    timer = undefined
  }
  function show(el: HTMLElement) {
    clear()
    anchor.value = el
    open.value = true
  }
  function hide() {
    clear()
    open.value = false
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') hide()
  }
  watch(open, isOpen => {
    if (isOpen) window.addEventListener('keydown', onKey, { capture: true })
    else window.removeEventListener('keydown', onKey, { capture: true })
  })
  onBeforeUnmount(() => {
    hide()
    window.removeEventListener('keydown', onKey, { capture: true })
  })

  const on = {
    pointerenter(e: PointerEvent) {
      if (e.pointerType === 'touch' || !(e.currentTarget instanceof HTMLElement)) return
      const el = e.currentTarget
      clear()
      timer = setTimeout(() => show(el), TOOLTIP_DELAY_MS)
    },
    pointerleave: hide,
    pointerdown() {
      hide()
      pressed = true
      window.addEventListener('pointerup', () => setTimeout(() => (pressed = false), 1), {
        once: true,
      })
    },
    focus(e: FocusEvent) {
      if (pressed || !(e.currentTarget instanceof HTMLElement)) return
      if (focusShowsTooltip(e.currentTarget)) show(e.currentTarget)
    },
    blur: hide,
    click: hide,
  }

  return { open, anchor, on, hide }
}
