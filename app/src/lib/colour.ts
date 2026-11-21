// Colour arithmetic for the interface: OKLCH (for lightness caps and accent shades that keep
// their hue) and the WCAG contrast ratio (for the 4.5:1 minimum on text). Pure, no DOM.

export interface Oklch {
  l: number
  c: number
  h: number
}
type Rgb = [number, number, number]

/** `#rrggbb` (or `#rgb`) → sRGB 0..1; anything else gives black. */
export function parseHex(hex: string): Rgb {
  let s = hex.trim().replace('#', '')
  if (s.length === 3) s = [...s].map(c => c + c).join('')
  const n = Number.parseInt(s.slice(0, 6), 16)
  if (s.length < 6 || Number.isNaN(n)) return [0, 0, 0]
  return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255]
}

export function toHex([r, g, b]: Rgb): string {
  const h = (v: number) =>
    Math.round(Math.min(1, Math.max(0, v)) * 255)
      .toString(16)
      .padStart(2, '0')
  return `#${h(r)}${h(g)}${h(b)}`
}

const toLinear = (v: number) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4)
const toGamma = (v: number) => (v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055)

function linearToOklab([r, g, b]: Rgb): Rgb {
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b)
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b)
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b)
  return [
    0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  ]
}

function oklabToLinear([L, a, b]: Rgb): Rgb {
  const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3
  const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3
  const s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3
  return [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ]
}

export function hexToOklch(hex: string): Oklch {
  const [L, a, b] = linearToOklab(parseHex(hex).map(toLinear) as Rgb)
  return { l: L, c: Math.hypot(a, b), h: ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360 }
}

const inGamut = (rgb: Rgb) => rgb.every(v => v >= -1e-4 && v <= 1 + 1e-4)

/** OKLCH → `#rrggbb`; out of the sRGB gamut, the chroma is reduced until it fits (hue kept). */
export function oklchToHex({ l, c, h }: Oklch): string {
  const rad = (h * Math.PI) / 180
  let chroma = c
  let lin = oklabToLinear([l, chroma * Math.cos(rad), chroma * Math.sin(rad)])
  while (!inGamut(lin) && chroma > 0) {
    chroma = Math.max(0, chroma - 0.002)
    lin = oklabToLinear([l, chroma * Math.cos(rad), chroma * Math.sin(rad)])
  }
  return toHex(lin.map(v => toGamma(Math.min(1, Math.max(0, v)))) as Rgb)
}

/** WCAG relative luminance. */
export function luminance(hex: string): number {
  const [r, g, b] = parseHex(hex)
  return 0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b)
}

/** WCAG contrast ratio of two opaque colours (1..21). */
export function contrast(a: string, b: string): number {
  const [la, lb] = [luminance(a), luminance(b)]
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05)
}

/** `top` at `alpha` (0..1) over the opaque `bottom`, in sRGB as browsers blend. */
export function over(top: string, alpha: number, bottom: string): string {
  const [tr, tg, tb] = parseHex(top)
  const [br, bg, bb] = parseHex(bottom)
  const mix = (t: number, b: number) => t * alpha + b * (1 - alpha)
  return toHex([mix(tr, br), mix(tg, bg), mix(tb, bb)])
}

/** The colour with its OKLCH lightness at most `max` (cover colours: ≤ 0.72, design system). */
export function capLightness(hex: string, max: number): string {
  const c = hexToOklch(hex)
  return c.l <= max ? toHex(parseHex(hex)) : oklchToHex({ ...c, l: max })
}

/** The colour with its OKLCH lightness moved by `delta` (hue and chroma kept where they fit). */
export function shiftLightness(hex: string, delta: number): string {
  const c = hexToOklch(hex)
  return oklchToHex({ ...c, l: Math.min(1, Math.max(0, c.l + delta)) })
}

/**
 * Moves the lightness of `hex` in `direction` (+1 lighter, −1 darker) in small steps until its
 * contrast with every colour of `against` reaches `min`; the hue stays. Returns the first that
 * passes (or the end of the range, which always passes for 4.5 against a theme's surfaces).
 */
export function untilContrast(
  hex: string,
  against: string[],
  min: number,
  direction: 1 | -1,
): string {
  const c = hexToOklch(hex)
  for (let l = c.l; l >= 0 && l <= 1; l += 0.005 * direction) {
    const out = oklchToHex({ ...c, l })
    if (against.every(bg => contrast(out, bg) >= min)) return out
  }
  return direction > 0 ? '#ffffff' : '#000000'
}
