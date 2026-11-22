// «Sfondo ai lati del gioco» (`video.surround*`, Settings board › Video): what fills the window
// around the game. The webview paints it: under the native video it is a full-window layer with a
// hole where the game is (the native surface shows through), in webview mode it sits under the
// canvas. Everything here is EmuDome's own: CSS gradients and a generated grid, no images.

/** The first built-in background, and the one used for an unknown `video.surroundSet`. */
export const AURORA = 'linear-gradient(135deg, #5b3f8c 0%, #245f7a 100%)'

/** The built-in backgrounds (`video.surroundSet`), exactly as the board's cards draw them. */
export const BUILTIN_BACKGROUNDS: Record<string, string> = {
  aurora: AURORA,
  sunset: 'linear-gradient(135deg, #8a3b2f 0%, #7a2f5a 100%)',
  night: 'linear-gradient(135deg, #0f1a33 0%, #1f3f6b 100%)',
  forest: 'linear-gradient(135deg, #1c3324 0%, #3d6b3a 100%)',
  graphite: 'linear-gradient(135deg, #16181e 0%, #2b2f38 100%)',
  grid: 'repeating-linear-gradient(0deg, #ffffff0d 0 1px, transparent 1px 16px), repeating-linear-gradient(90deg, #ffffff0d 0 1px, transparent 1px 16px), #16181e',
}

/** The preview of each `video.surround` card (board), shown in the settings pages. */
export const SURROUND_CARDS = {
  black: '#000',
  cover: 'radial-gradient(70% 90% at 30% 20%, #2f5d8a 0%, #12202e 70%)',
  blur: 'radial-gradient(60% 80% at 50% 40%, #5f8fc4 0%, #3d6b3a 60%, #1e2129 100%)',
  builtin: AURORA,
  image: 'repeating-linear-gradient(45deg, #1e2129 0 8px, #262a33 8px 16px)',
}

/** «Colore della copertina»: the board card's gradient with the cover's colours. */
export function coverBackground(light: string, dark: string): string {
  return `radial-gradient(70% 90% at 30% 20%, ${light} 0%, ${dark} 70%)`
}

export interface Rect {
  x: number
  y: number
  w: number
  h: number
}

/** A rectangle as its edges. */
interface Box {
  x1: number
  y1: number
  x2: number
  y2: number
}

const overlap = (a: Box, b: Box) => a.x1 < b.x2 && b.x1 < a.x2 && a.y1 < b.y2 && b.y1 < a.y2

/**
 * The holes to cut: rectangles that overlap (a custom layout may put one screen over another)
 * are merged into their bounding box, since `evenodd` would paint their overlap back. At worst
 * the native black shows in a corner between two overlapping screens.
 */
function disjoint(boxes: Box[]): Box[] {
  const out = [...boxes]
  for (let merged = true; merged;) {
    merged = false
    outer: for (let i = 0; i < out.length; i++)
      for (let j = i + 1; j < out.length; j++) {
        const [a, b] = [out[i], out[j]]
        if (!a || !b || !overlap(a, b)) continue
        out[i] = {
          x1: Math.min(a.x1, b.x1),
          y1: Math.min(a.y1, b.y1),
          x2: Math.max(a.x2, b.x2),
          y2: Math.max(a.y2, b.y2),
        }
        out.splice(j, 1)
        merged = true
        break outer
      }
  }
  return out
}

/**
 * The CSS `clip-path` that keeps everything but `holes` (each screen where the native video draws
 * it, from Rust's `screens_placements`; in the layer's CSS px, `scale` device pixels per layer
 * px), each rounded outwards to whole device pixels so the background never covers a screen's
 * edge: at worst a pixel of the native black shows. One path: the window, then each hole joined
 * to its corner and back (the joins cancel out under `evenodd`). No holes: undefined.
 */
export function holesClipPath(holes: readonly Rect[], scale: number): string | undefined {
  // The small margin absorbs float noise (800 / 1.2 * 1.2 is not exactly 800).
  const boxes = holes
    .filter(h => h.w > 0 && h.h > 0)
    .map(h => ({
      x1: Math.floor(h.x * scale + 1e-6) / scale,
      y1: Math.floor(h.y * scale + 1e-6) / scale,
      x2: Math.ceil((h.x + h.w) * scale - 1e-6) / scale,
      y2: Math.ceil((h.y + h.h) * scale - 1e-6) / scale,
    }))
  if (boxes.length === 0) return undefined
  const p = (x: number, y: number) => `${x}px ${y}px`
  const rings = disjoint(boxes).map(
    b =>
      `${p(b.x1, b.y1)}, ${p(b.x2, b.y1)}, ${p(b.x2, b.y2)}, ${p(b.x1, b.y2)}, ${p(b.x1, b.y1)}, 0 0`,
  )
  return `polygon(evenodd, 0 0, 100% 0, 100% 100%, 0 100%, 0 0, ${rings.join(', ')})`
}
