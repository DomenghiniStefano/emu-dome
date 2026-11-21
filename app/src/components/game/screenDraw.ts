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
