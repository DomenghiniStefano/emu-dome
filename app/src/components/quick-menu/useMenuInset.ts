// The quick menu's part of the window kept clear of the game (#29): the phone's sheet or side
// panel measured where it is, again whenever it moves or changes size, the window is resized or
// the phone turned, so the game is laid out beside the panel and never left where the other way
// round put it. The desktop's G09 bar measures its own (ScreensBar), and C06 sends its own
// (TouchEditor).
import { nextTick, onBeforeUnmount, onMounted, watch, type Ref } from 'vue'
import { NO_INSET, type InsetPx } from '@/components/game/inset'
import { MenuInset, panelInset } from './panel'

export function useMenuInset(
  panel: Ref<HTMLElement | null>,
  kind: () => MenuInset,
  emit: (inset: InsetPx) => void,
) {
  let observer: ResizeObserver | null = null

  function report() {
    const k = kind()
    if (k === MenuInset.Own) return
    const el = panel.value
    if (!el || k === MenuInset.None) return emit(NO_INSET)
    const r = el.getBoundingClientRect()
    emit(panelInset(k, r, { width: window.innerWidth, height: window.innerHeight }))
  }
  /** After the panel's classes for the new layout or page are applied. */
  const later = () => void nextTick(report)

  function observe(el: HTMLElement | null) {
    observer?.disconnect()
    if (el && typeof ResizeObserver !== 'undefined') {
      observer = new ResizeObserver(report)
      observer.observe(el)
    }
  }

  watch(panel, el => {
    observe(el)
    later()
  })
  watch(kind, later)
  onMounted(() => {
    window.addEventListener('resize', later)
    observe(panel.value)
    later()
  })
  onBeforeUnmount(() => {
    window.removeEventListener('resize', later)
    observer?.disconnect()
    observer = null
  })
}
