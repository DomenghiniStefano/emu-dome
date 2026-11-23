// `general.accent` (T02 «Colore d’accento»): the swatches of the schema, each turned into the
// accent tokens of colors.css for every theme. The default swatch (amber, the app icon's) is
// written in colors.css; every other swatch is computed here by the same rules, and
// accent.test.ts holds the default's values in colors.css to them:
// - the fill is the swatch, and the text on it (`on-accent`) is white or DARK_ON_ACCENT, whichever
//   reads at 4.5:1 on the fill and on its hover shade (white first); only if neither does is the
//   fill darkened until white does;
// - hover, deep and crown take the shape of the design system's first ramp (VIOLET_RAMP: the
//   same OKLCH lightness steps from the fill), except the default's deep and crown, which are the
//   icon's own (design, colors.css);
// - `accent-text` and `focus-ring` are moved in lightness per theme until they reach 4.5:1 on
//   every surface of that theme, including a selected chip (`accent-soft` over `surface-raised`).
// Pure: appearance.ts writes the result into the page.
import catalog from '@/generated/settings-catalog.json'
import { contrast, hexToOklch, oklchToHex, over, untilContrast } from './colour'
import type { Theme } from '@/emu'

/** `general.accent` in the settings catalogue (Rust, generated): its default and its swatches. */
const ENTRY = catalog.colours['general.accent']

/** The default swatch of `general.accent`, whose tokens colors.css already holds. */
export const DEFAULT_ACCENT = ENTRY.default
/** The swatches of `general.accent`, in the schema's order. */
export const ACCENT_SWATCHES: readonly string[] = ENTRY.swatches

/**
 * The shape every swatch's shades take: the design system's first accent (GameDome's violet) and
 * its hover, deep and crown, read as OKLCH lightness steps from the fill. Only the lightness is
 * taken: the violet swatch itself gets shades close to, not equal to, these.
 */
const VIOLET_RAMP = { fill: '#7152f7', hover: '#6544ee', deep: '#5a3fdc', crown: '#8a70ff' }

/** Text contrast the design system asks for (README «Colore»). */
export const MIN_TEXT = 4.5
const WHITE = '#ffffff'
/** Text on a light accent (amber): the Scuro ground (`--bg`). */
export const DARK_ON_ACCENT = '#0e0f13'

/** The surfaces accent text sits on, per theme (colors.css), and the alpha of `accent-soft`. */
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

/** Shade of `fill` at one step of VIOLET_RAMP (hue and chroma kept where they fit). */
function shade(fill: string, like: string): string {
  const d = hexToOklch(like).l - hexToOklch(VIOLET_RAMP.fill).l
  const c = hexToOklch(fill)
  return oklchToHex({ ...c, l: Math.min(1, Math.max(0, c.l + d)) })
}

/** The hover fill of a main button whose fill is `fill`. */
export const accentHover = (fill: string) => shade(fill, VIOLET_RAMP.hover)

/** Whether `text` reads at 4.5:1 on `fill` and on its hover shade (a main button's two fills). */
function readsOn(text: string, fill: string): boolean {
  return [fill, accentHover(fill)].every(bg => contrast(text, bg) >= MIN_TEXT)
}

/**
 * A main button's fill and the text on it: the swatch kept, with white or DARK_ON_ACCENT text,
 * whichever reads (white first); if neither does, the swatch darkened until white text reads on
 * it (and so on its darker hover).
 */
export function accentFill(swatch: string): { fill: string; on: string } {
  for (const on of [WHITE, DARK_ON_ACCENT]) if (readsOn(on, swatch)) return { fill: swatch, on }
  return { fill: untilContrast(swatch, [WHITE], MIN_TEXT, -1), on: WHITE }
}

/** `accent-text` for one theme: readable on every surface and on a selected chip. */
export function accentText(swatch: string, theme: Theme): string {
  const s = SURFACES[theme]
  const grounds = [...s.list, over(swatch, s.soft, s.raised)]
  return untilContrast(swatch, grounds, MIN_TEXT, theme === 'light' ? -1 : 1)
}

/** Every accent token of `hex` (`#rrggbb`, lowercase), per theme, by the rules above. */
export function computeAccentTokens(hex: string): Record<Theme, AccentTokens> {
  const { fill, on } = accentFill(hex)
  const fills: AccentTokens = {
    '--violet': hex,
    '--accent': fill,
    '--accent-hover': accentHover(fill),
    '--accent-deep': shade(fill, VIOLET_RAMP.deep),
    '--accent-crown': shade(fill, VIOLET_RAMP.crown),
    '--on-accent': on,
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

/**
 * The tokens to override for `swatch`, per theme; null for the default accent (colors.css already
 * holds its values) or a value that is not a colour.
 */
export function accentTokens(swatch: string): Record<Theme, AccentTokens> | null {
  const hex = swatch.trim().toLowerCase()
  if (hex === DEFAULT_ACCENT || !/^#[0-9a-f]{6}$/.test(hex)) return null
  return computeAccentTokens(hex)
}
