// The webview canvas (GameCanvas) draws the screens where Rust placed them
// (`emudome_core::screens::compose`, through `emu.screens.placements`): no geometry is decided
// here, each placement only becomes the canvas calls that draw it.
import type { ScreenPlacement, ScreenRotation } from '@/emu'

/** One `drawImage` of a screen: its part of the frame, and where it lands, turned. */
export interface ScreenDraw {
  /** The screen's pixels in the frame. */
  sx: number
  sy: number
  sw: number
  sh: number
  /** The centre of its rectangle on the canvas, in canvas pixels. */
  cx: number
  cy: number
  /** The turn around that centre, radians (clockwise on the canvas). */
  angle: number
  /** The picture's size before the turn, in canvas pixels. */
  w: number
  h: number
}

const ANGLE: Record<ScreenRotation, number> = {
  none: 0,
  // A quarter turn anticlockwise.
  left: -Math.PI / 2,
  right: Math.PI / 2,
  flip: Math.PI,
}

/** How one screen is smoothed on the canvas (`video.filter`, as the native renderer's shader
 * does): `smooth` off draws the frame's pixels sharp; on, the screen is first enlarged sharp by
 * `kx` × `ky` (whole numbers, along the frame's axes) into a picture of its own, which is then
 * smoothed to the rest of the size. That own picture also keeps the other screen of the frame
 * from bleeding in at the edges. */
export interface ScreenSmoothing {
  smooth: boolean
  kx: number
  ky: number
}

/** `none`: sharp pixels. `light` («sharp bilinear»): enlarged by the largest whole scale that
 * fits, smoothed only for the rest, so every pixel has the same size. `soft`: smoothed all over.
 * Another value is the catalogue's default, `light`. */
export function screenSmoothing(d: ScreenDraw, smoothing: string): ScreenSmoothing {
  if (smoothing === 'none') return { smooth: false, kx: 1, ky: 1 }
  if (smoothing === 'soft') return { smooth: true, kx: 1, ky: 1 }
  // A scale such as 4 may come as 3.9999…: still 4.
  const whole = (scale: number) => Math.max(1, Math.floor(scale + 1e-6))
  return { smooth: true, kx: whole(d.w / d.sw), ky: whole(d.h / d.sh) }
}

/** The draws of `placements` (window units, as Rust composed them) on a canvas `scale` times
 * finer (the device pixel ratio). */
export function screenDraws(placements: ScreenPlacement[], scale: number): ScreenDraw[] {
  return placements.map(({ src, dst, rotation }) => {
    const turned = rotation === 'left' || rotation === 'right'
    return {
      sx: src.x,
      sy: src.y,
      sw: src.w,
      sh: src.h,
      cx: (dst.x + dst.w / 2) * scale,
      cy: (dst.y + dst.h / 2) * scale,
      angle: ANGLE[rotation],
      w: (turned ? dst.h : dst.w) * scale,
      h: (turned ? dst.w : dst.h) * scale,
    }
  })
}
