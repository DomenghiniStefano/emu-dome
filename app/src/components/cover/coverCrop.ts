// «Adatta la copertina» (Trello #85): the geometry of framing an image in the library's 2:3
// cover. Pure, tested (coverCrop.test.ts). The interface decides only the view, the part of the
// image the cover shows, in source pixels (`CoverView`); Rust makes the cover from it
// (`emudome_core::cover`) and draws the spare space the same way the preview does: the image
// filling the frame, blurred and dimmed.
//
// A crop is a mode, a zoom and the view's centre:
// - «Intera» (whole): at zoom 1 the whole image fits the frame (the rest is the backdrop); the
//   centre stays on the image, so some of it is always in the frame.
// - «Riempi» (fill): at zoom 1 the image fills the frame (its middle cropped); the view never
//   leaves the image, so the frame is always the image's own pixels.
import type { CoverView } from '@/emu'

/** The cover's shape, width : height (Rust's 600 × 900). */
const SHAPE_W = 2
const SHAPE_H = 3
/** The cover's shape, width over height. */
export const COVER_ASPECT = SHAPE_W / SHAPE_H

export const CropMode = {
  Whole: 'whole',
  Fill: 'fill',
} as const
export type CropMode = (typeof CropMode)[keyof typeof CropMode]

export const ZOOM_MIN = 1
export const ZOOM_MAX = 4
/** One step of the triggers, the keys or the slider. */
export const ZOOM_STEP = 0.1
/** One d-pad or arrow step: this share of the frame. */
export const PAN_STEP = 0.05

export interface ImageSize {
  width: number
  height: number
}

export interface Crop {
  mode: CropMode
  zoom: number
  /** The view's centre, in source pixels. */
  cx: number
  cy: number
}

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v))

/** The view's size at zoom 1 for `mode`. */
export function baseSize(img: ImageSize, mode: CropMode): ImageSize {
  // Compared and scaled in integers' terms, so a 2:3 image gives exact numbers.
  const wider = img.width * SHAPE_H > img.height * SHAPE_W
  // Whole: the image's long side (for the frame) fits; fill: its short side does.
  const byWidth = mode === CropMode.Whole ? wider : !wider
  return byWidth
    ? { width: img.width, height: (img.width * SHAPE_H) / SHAPE_W }
    : { width: (img.height * SHAPE_W) / SHAPE_H, height: img.height }
}

/** The view's size for a crop. */
function viewSize(img: ImageSize, crop: Pick<Crop, 'mode' | 'zoom'>): ImageSize {
  const base = baseSize(img, crop.mode)
  return { width: base.width / crop.zoom, height: base.height / crop.zoom }
}

/** The crop with its zoom in range and its centre where its mode allows. */
export function clampCrop(img: ImageSize, crop: Crop): Crop {
  const zoom = clamp(crop.zoom, ZOOM_MIN, ZOOM_MAX)
  if (crop.mode === CropMode.Whole) {
    return {
      ...crop,
      zoom,
      cx: clamp(crop.cx, 0, img.width),
      cy: clamp(crop.cy, 0, img.height),
    }
  }
  const size = viewSize(img, { mode: crop.mode, zoom })
  const halfW = Math.min(size.width, img.width) / 2
  const halfH = Math.min(size.height, img.height) / 2
  return {
    ...crop,
    zoom,
    cx: clamp(crop.cx, halfW, img.width - halfW),
    cy: clamp(crop.cy, halfH, img.height - halfH),
  }
}

/** Where the step starts: the whole image, centred. */
export function initialCrop(img: ImageSize, mode: CropMode = CropMode.Whole): Crop {
  return { mode, zoom: ZOOM_MIN, cx: img.width / 2, cy: img.height / 2 }
}

/** The view Rust makes the cover from. */
export function viewOf(img: ImageSize, crop: Crop): CoverView {
  const c = clampCrop(img, crop)
  const size = viewSize(img, c)
  return {
    x: c.cx - size.width / 2,
    y: c.cy - size.height / 2,
    width: size.width,
    height: size.height,
  }
}

/** Another mode: zoom 1 again, the centre kept where the mode allows. */
export function withMode(img: ImageSize, crop: Crop, mode: CropMode): Crop {
  return clampCrop(img, { ...crop, mode, zoom: ZOOM_MIN })
}

/** Another zoom, the centre kept. */
export function withZoom(img: ImageSize, crop: Crop, zoom: number): Crop {
  return clampCrop(img, { ...crop, zoom: Math.round(zoom * 100) / 100 })
}

/** One zoom step in or out (`dir` 1 / -1). */
export function zoomStep(img: ImageSize, crop: Crop, dir: 1 | -1): Crop {
  return withZoom(img, crop, crop.zoom + dir * ZOOM_STEP)
}

/**
 * Moves the image by a share of the frame (`dx`, `dy`: 1 = the frame's whole width / height),
 * as a drag does: the image goes the way the finger goes, so the view goes the other way.
 */
export function moveBy(img: ImageSize, crop: Crop, dx: number, dy: number): Crop {
  const size = viewSize(img, crop)
  return clampCrop(img, { ...crop, cx: crop.cx - dx * size.width, cy: crop.cy - dy * size.height })
}

export type Direction = 'up' | 'down' | 'left' | 'right'

/** One d-pad or arrow step: the image moves that way by [`PAN_STEP`] of the frame. */
export function panStep(img: ImageSize, crop: Crop, dir: Direction): Crop {
  const dx = dir === 'left' ? -PAN_STEP : dir === 'right' ? PAN_STEP : 0
  const dy = dir === 'up' ? -PAN_STEP : dir === 'down' ? PAN_STEP : 0
  return moveBy(img, crop, dx, dy)
}

/** Where the image sits in the frame, in % of the frame (the preview's placement). */
export interface Placement {
  left: number
  top: number
  width: number
  height: number
}

export function placement(img: ImageSize, view: CoverView): Placement {
  return {
    left: ((0 - view.x) / view.width) * 100,
    top: ((0 - view.y) / view.height) * 100,
    width: (img.width / view.width) * 100,
    height: (img.height / view.height) * 100,
  }
}

/** The zoom as a percentage, for the slider's label. */
export const zoomPercent = (zoom: number) => Math.round(zoom * 100)
