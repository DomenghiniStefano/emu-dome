// `general.accent` (T02 «Colore d’accento»): the five swatches of the schema, each turned into the
// accent tokens of styles.css for every theme. The default swatch keeps the design system's
// values exactly; the others take the same shape in OKLCH (same lightness steps from the fill to
// hover, deep and crown), then:
// - the fill is darkened until white text on it (`on-accent`) reaches 4.5:1;
// - `accent-text` and `focus-ring` are moved in lightness per theme until they reach 4.5:1 on
//   every surface of that theme, including a selected chip (`accent-soft` over `surface-raised`).
// Pure: appearance.ts writes the result into the page.
import { contrast, hexToOklch, oklchToHex, over, untilContrast } from './colour'
import type { Theme } from './emu'

/** The design system's accent (styles.css :root), the default swatch of `general.accent`. */
export const DEFAULT_ACCENT = '#7152f7'
const BASE = { hover: '#6544ee', deep: '#5a3fdc', crown: '#8a70ff' }

/** Text contrast the design system asks for (README «Colore»). */
export const MIN_TEXT = 4.5
const WHITE = '#ffffff'

/** The surfaces accent text sits on, per theme (styles.css), and the alpha of `accent-soft`. */
export const SURFACES: Record<Theme, { list: string[]; raised: string; soft: number }> = {
  dark: { list: ['#0e0f13', '#16181e', '#1e2129', '#262a33'], raised: '#1e2129', soft: 0x2e / 255 },
  black: {
    list: ['#000000', '#0b0c0f', '#15161b', '#1d1f25'],
    raised: '#15161b',
    soft: 0x2e / 255,
  },
  light: {
    list: ['#f4f3f0', '#ffffff', '#f7f6f3', '#efeee9'],
    raised: '#f7f6f3',
    soft: 0x1f / 255,
  },
}

export type AccentTokens = Record<string, string>

/** Shades of `fill` at the default accent's lightness steps (hue and chroma kept where they fit). */
function shade(fill: string, like: string): string {
  const d = hexToOklch(like).l - hexToOklch(DEFAULT_ACCENT).l
  const c = hexToOklch(fill)
  return oklchToHex({ ...c, l: Math.min(1, Math.max(0, c.l + d)) })
}

/** The fill used for main buttons: the swatch, darkened until white text on it reads at 4.5:1. */
export function accentFill(swatch: string): string {
  return contrast(WHITE, swatch) >= MIN_TEXT ? swatch : untilContrast(swatch, [WHITE], MIN_TEXT, -1)
}

/** `accent-text` for one theme: readable on every surface and on a selected chip. */
export function accentText(swatch: string, theme: Theme): string {
  const s = SURFACES[theme]
  const grounds = [...s.list, over(swatch, s.soft, s.raised)]
  return untilContrast(swatch, grounds, MIN_TEXT, theme === 'light' ? -1 : 1)
}

/**
 * The tokens to override for `swatch`, per theme; null for the default accent (styles.css already
 * holds the design system's values) or a value that is not a colour.
 */
export function accentTokens(swatch: string): Record<Theme, AccentTokens> | null {
  const hex = swatch.trim().toLowerCase()
  if (hex === DEFAULT_ACCENT || !/^#[0-9a-f]{6}$/.test(hex)) return null
  const fill = accentFill(hex)
  const fills: AccentTokens = {
    '--violet': hex,
    '--accent': fill,
    '--accent-hover': shade(fill, BASE.hover),
    '--accent-deep': shade(fill, BASE.deep),
    '--accent-crown': shade(fill, BASE.crown),
    '--on-accent': WHITE,
  }
  const per = (theme: Theme): AccentTokens => {
    const text = accentText(hex, theme)
    const alpha = Math.round(SURFACES[theme].soft * 255)
      .toString(16)
      .padStart(2, '0')
    return {
      ...fills,
      '--accent-text': text,
      '--focus-ring': text,
      '--accent-soft': `${hex}${alpha}`,
    }
  }
  return { dark: per('dark'), black: per('black'), light: per('light') }
}
