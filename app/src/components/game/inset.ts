// The edges of the window kept clear of the game (#29): the quick menu's bar, sheet or side
// panel, the touch controls' band upright or their columns on the side. The interface measures
// them in CSS pixels; Rust takes shares of the window (`VideoInset`, `emudome_core::screens::Inset`)
// and fits the game in what is left (`screens::fit`), the one rule of both renderers.
import type { VideoAlign, VideoInset } from '@/emu'

/** The edges kept clear, in CSS pixels of the window, where the game sits in the rest, and the
 * room kept between two screens (the touch controls' strip on a phone, `Inset::between`). */
export interface InsetPx {
  left: number
  top: number
  right: number
  bottom: number
  align: VideoAlign
  between: number
}

/** Nothing kept clear: the whole window. */
export const NO_INSET: InsetPx = Object.freeze({
  left: 0,
  top: 0,
  right: 0,
  bottom: 0,
  align: 'centre',
  between: 0,
})

/** The bottom `px` kept clear, the game centred above (the desktop's G09 bar). */
export const bottomInset = (px: number): InsetPx => ({ ...NO_INSET, bottom: px })

/** The two insets keep the same edges, the same way. */
export const sameInset = (a: InsetPx, b: InsetPx) =>
  a.left === b.left &&
  a.top === b.top &&
  a.right === b.right &&
  a.bottom === b.bottom &&
  a.align === b.align &&
  a.between === b.between

/** The most of the window one edge may keep (Rust also keeps a tenth of each axis free). */
const MAX_SHARE = 0.9
/** The most of the window's height the room between two screens may take (`BETWEEN_MAX`). */
const MAX_BETWEEN = 0.5

/** `px` as shares of a `width` × `height` window (each in 0..0.9; an empty window keeps none).
 * The room between two screens is a share of the height whatever the axis, 0.5 at most. */
export function insetShares(px: InsetPx, width: number, height: number): VideoInset {
  const share = (v: number, whole: number, max = MAX_SHARE) =>
    whole > 0 && Number.isFinite(v) ? Math.min(Math.max(v / whole, 0), max) : 0
  return {
    left: share(px.left, width),
    top: share(px.top, height),
    right: share(px.right, width),
    bottom: share(px.bottom, height),
    align: px.align,
    between: share(px.between, height, MAX_BETWEEN),
  }
}
