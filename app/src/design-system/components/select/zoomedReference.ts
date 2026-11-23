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

/**
 * A length token (`--spacing-8`) in CSS px, for the offsets Floating UI takes as numbers (a
 * popover's or a menu's distance from its button), so they stay design tokens.
 */
export function tokenPx(name: string): number {
  return parseFloat(getComputedStyle(document.documentElement).getPropertyValue(name)) || 0
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

/**
 * The window as a Floating UI collision boundary in the zoomed page's CSS px. Floating UI's own
 * viewport is `html.clientWidth` / `clientHeight`, in screen px whatever the zoom: at 120% a
 * popover under its button believed it had the window's 360 px of room where the page has 300,
 * and its available height (`--reka-popover-content-available-height`) ran past the window's
 * bottom (#106). Floating UI takes a plain rectangle as a boundary; its getters are read at each
 * placement, so a resized window or a new scale is followed. At 100% it is the viewport itself.
 */
export function zoomedViewport() {
  return {
    x: 0,
    y: 0,
    get width() {
      return window.innerWidth / pageZoom()
    },
    get height() {
      return window.innerHeight / pageZoom()
    },
  }
}
