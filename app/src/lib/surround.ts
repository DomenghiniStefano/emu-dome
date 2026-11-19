// «Sfondo ai lati del gioco» (`video.surround*`, Settings board › Video): what fills the window
// around the game. The webview paints it: under the native video it is a full-window layer with a
// hole where the game is (the native surface shows through), in webview mode it sits under the
// canvas. Everything here is EmuDome's own: CSS gradients and a generated grid, no images.

/** The built-in backgrounds (`video.surroundSet`), exactly as the board's cards draw them. */
export const BUILTIN_BACKGROUNDS: Record<string, string> = {
  aurora: 'linear-gradient(135deg, #5b3f8c 0%, #245f7a 100%)',
  sunset: 'linear-gradient(135deg, #8a3b2f 0%, #7a2f5a 100%)',
  night: 'linear-gradient(135deg, #0f1a33 0%, #1f3f6b 100%)',
  forest: 'linear-gradient(135deg, #1c3324 0%, #3d6b3a 100%)',
  graphite: 'linear-gradient(135deg, #16181e 0%, #2b2f38 100%)',
  grid: 'repeating-linear-gradient(0deg, #ffffff0d 0 1px, transparent 1px 16px), repeating-linear-gradient(90deg, #ffffff0d 0 1px, transparent 1px 16px), #16181e',
}

/** The preview of each `video.surround` card (board), shown in the settings pages. */
export const SURROUND_CARDS: Record<string, string> = {
  black: '#000',
  cover: 'radial-gradient(70% 90% at 30% 20%, #2f5d8a 0%, #12202e 70%)',
  blur: 'radial-gradient(60% 80% at 50% 40%, #5f8fc4 0%, #3d6b3a 60%, #1e2129 100%)',
  builtin: BUILTIN_BACKGROUNDS.aurora,
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

/**
 * Where the native video draws the game, in physical pixels: a mirror of `viewport` in
 * app/src-tauri/src/video.rs (integer scale when it fits, else the largest that keeps the aspect,
 * centred above the bottom `inset` share the quick menu's bar keeps clear). Keep the two equal.
 */
export function nativeGameRect(
  surface: { w: number; h: number },
  game: { w: number; h: number },
  inset: number,
): Rect {
  const sh = surface.h * (1 - Math.min(0.9, Math.max(0, inset)))
  const fit = Math.min(surface.w / game.w, sh / game.h)
  const scale = fit >= 1 ? Math.floor(fit) : fit
  const w = game.w * scale
  const h = game.h * scale
  return { x: Math.floor((surface.w - w) / 2), y: Math.floor((sh - h) / 2), w, h }
}

/**
 * The CSS `clip-path` that keeps everything but `hole` (in the layer's CSS px; `scale` device
 * pixels per layer px), rounded outwards to whole device pixels so the background never covers
 * the game's edge: at worst a pixel of the native black shows.
 */
export function holeClipPath(hole: Rect, scale: number): string {
  const px = (v: number) => `${v}px`
  // The small margin absorbs float noise (800 / 1.2 * 1.2 is not exactly 800).
  const x1 = Math.floor(hole.x * scale + 1e-6) / scale
  const y1 = Math.floor(hole.y * scale + 1e-6) / scale
  const x2 = Math.ceil((hole.x + hole.w) * scale - 1e-6) / scale
  const y2 = Math.ceil((hole.y + hole.h) * scale - 1e-6) / scale
  return (
    `polygon(evenodd, 0 0, 100% 0, 100% 100%, 0 100%, 0 0, ` +
    `${px(x1)} ${px(y1)}, ${px(x2)} ${px(y1)}, ${px(x2)} ${px(y2)}, ${px(x1)} ${px(y2)}, ${px(x1)} ${px(y1)})`
  )
}
