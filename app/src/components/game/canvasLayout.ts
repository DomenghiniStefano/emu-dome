// The webview mode's game size (GameCanvas): as big as the window allows above the quick menu's
// bar, by whole steps when it fits at least once (sharp pixels), else shrunk to fit. Sizes come
// from the window, in screen px; the interface may be scaled (`a11y.textsize`, CSS zoom on <html>)
// and the game must not be, so the CSS px written are the screen px divided by the zoom.

export interface Size {
  w: number
  h: number
}

/** The canvas's CSS size for a `game` frame in a `window`, `inset` px kept clear at the bottom. */
export function canvasSize(game: Size, window: Size, inset: number, zoom: number): Size | null {
  if (!game.w || !game.h) return null
  const height = Math.max(1, window.h - inset)
  const fit = Math.min(window.w / game.w, height / game.h)
  const scale = (fit >= 1 ? Math.floor(fit) : fit) / (zoom || 1)
  return { w: game.w * scale, h: game.h * scale }
}
