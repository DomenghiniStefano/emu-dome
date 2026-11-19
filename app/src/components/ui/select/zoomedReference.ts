// The interface's scale is CSS `zoom` on <html> (`a11y.textsize`, appearance.ts). Floating UI,
// which places Reka's popper content, measures the trigger with getBoundingClientRect (screen px,
// zoom included) and then writes that position as CSS px inside the zoomed page, where it is
// zoomed once more: at 110% a list opened 10% too far right and down, and 10% too wide (measured
// 2026-11-18). The anchor handed to it is the trigger in CSS px instead; at 100% it is the
// trigger's own rectangle.

/** The page's zoom on <html> (1 when none). */
export function pageZoom(): number {
  return parseFloat(getComputedStyle(document.documentElement).zoom) || 1
}

/** A Floating UI reference for `el` measured in the zoomed page's CSS px. */
export function zoomedReference(el: HTMLElement) {
  return {
    // Floating UI follows the scrolling ancestors of this element.
    contextElement: el,
    getBoundingClientRect(): DOMRect {
      const z = pageZoom()
      const r = el.getBoundingClientRect()
      return new DOMRect(r.x / z, r.y / z, r.width / z, r.height / z)
    },
  }
}
