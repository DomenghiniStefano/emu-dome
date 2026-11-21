// The DS touch screen from the mouse or a finger: what the interface reports. Only where the
// pointer is and in which area; Rust composes the screens and maps the point to the touch screen
// (`emudome_core::screens::pointer_at`), so the mapping lives in one tested place.
import type { PointerArea, VideoMode } from '@/emu'

/** What the pointer's position is measured against: the window, and the menu's inset. */
export interface Viewport {
  innerWidth: number
  innerHeight: number
  devicePixelRatio: number
}

/**
 * The area the screens are composed in, in the renderer's unit (`scale` per CSS pixel): the
 * window, `insetPx` CSS pixels at the bottom kept clear for the quick menu. The webview canvas
 * asks Rust for its placements in it (scale 1); the touch screen is measured against it.
 */
export function viewArea(view: Viewport, scale: number, insetPx: number): PointerArea {
  const inset = view.innerHeight > 0 ? insetPx / view.innerHeight : 0
  return {
    width: view.innerWidth * scale,
    height: view.innerHeight * scale,
    inset: Math.min(Math.max(inset, 0), 0.9),
  }
}

/**
 * The point and area in the renderer's unit: the native video draws in physical pixels, the
 * webview canvas in CSS pixels (each composes the screens in its own unit, in Rust).
 * `insetPx`: the CSS pixels at the bottom kept clear for the quick menu.
 */
export function pointerArgs(
  clientX: number,
  clientY: number,
  view: Viewport,
  mode: VideoMode,
  insetPx: number,
): { x: number; y: number; area: PointerArea } {
  const scale = mode === 'native' ? view.devicePixelRatio || 1 : 1
  return { x: clientX * scale, y: clientY * scale, area: viewArea(view, scale, insetPx) }
}

/** A press on an interface element (on-screen buttons, the d-pad, the menu) is not a touch. */
export function isInterfaceTarget(target: EventTarget | null): boolean {
  return (
    target instanceof Element &&
    target.closest('button, a, input, select, textarea, [role], [tabindex]') !== null
  )
}
