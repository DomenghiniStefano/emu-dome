import { describe, expect, it } from 'vitest'
import colors from '@/design-system/tokens/colors.css?raw'
import catalog from '@/generated/settings-catalog.json'
import {
  ACCENT_SWATCHES,
  accentFill,
  accentHover,
  accentText,
  accentTokens,
  computeAccentTokens,
  DARK_ON_ACCENT,
  DEFAULT_ACCENT,
  MIN_TEXT,
  SURFACES,
} from './accent'
import { contrast, over } from './colour'

const AMBER = '#faa40f'
const VIOLET = '#7152f7'
const THEMES = ['dark', 'black', 'light'] as const

/** The custom properties a colors.css block declares, by its selector. */
function block(selector: string): Record<string, string> {
  const start = colors.indexOf(`${selector} {`)
  const body = colors.slice(colors.indexOf('{', start) + 1, colors.indexOf('}', start))
  return Object.fromEntries(
    [...body.matchAll(/(--[\w-]+):\s*([^;]+);/g)].map(m => [m[1]!, m[2]!.trim()]),
  )
}

describe('the default accent', () => {
  it('comes from the settings schema', () => {
    expect(DEFAULT_ACCENT).toBe(catalog.colours['general.accent'].default)
    expect(DEFAULT_ACCENT).toBe(AMBER)
    expect(ACCENT_SWATCHES).toEqual([AMBER, VIOLET, '#2f7ae5', '#1f9d8a', '#c2417a'])
  })

  it('needs no override: colors.css holds it', () => {
    expect(accentTokens(DEFAULT_ACCENT)).toBeNull()
    expect(accentTokens(' #FAA40F ')).toBeNull()
  })

  it('is written in colors.css as the rules compute it, with the icon’s crown and deep', () => {
    const computed = computeAccentTokens(DEFAULT_ACCENT)
    const css = {
      dark: block(':root'),
      black: { ...block(':root'), ...block(":root[data-theme='black']") },
      light: { ...block(':root'), ...block(":root[data-theme='light']") },
    }
    for (const theme of THEMES) {
      const { ['--accent-crown']: _crown, ['--accent-deep']: _deep, ...ruled } = computed[theme]
      for (const [name, value] of Object.entries(ruled))
        expect(`${theme} ${name} ${css[theme][name]}`).toBe(`${theme} ${name} ${value}`)
      expect(css[theme]['--accent-crown']).toBe('#ffc847')
      expect(css[theme]['--accent-deep']).toBe('#cd860c')
    }
  })
})

describe('text on the accent', () => {
  it('is dark on amber, which keeps its colour', () => {
    expect(accentFill(AMBER)).toEqual({ fill: AMBER, on: DARK_ON_ACCENT })
    expect(contrast(DARK_ON_ACCENT, AMBER)).toBeGreaterThanOrEqual(MIN_TEXT)
    expect(contrast('#ffffff', AMBER)).toBeLessThan(MIN_TEXT)
  })

  it('is white on violet, which keeps its colour', () => {
    expect(accentFill(VIOLET)).toEqual({ fill: VIOLET, on: '#ffffff' })
  })

  it('is white on a darkened fill only when neither white nor dark reads on the swatch', () => {
    // Blue: white is 4.2:1 on it, dark 4.6:1 on it but under 4.5:1 on its hover.
    const blue = accentFill('#2f7ae5')
    expect(blue.on).toBe('#ffffff')
    expect(blue.fill).not.toBe('#2f7ae5')
    expect(contrast('#ffffff', blue.fill)).toBeGreaterThanOrEqual(MIN_TEXT)
  })

  it('reads at 4.5:1 on the fill and on its hover for every swatch', () => {
    for (const swatch of ACCENT_SWATCHES) {
      const { fill, on } = accentFill(swatch)
      expect(contrast(on, fill), swatch).toBeGreaterThanOrEqual(MIN_TEXT)
      expect(contrast(on, accentHover(fill)), swatch).toBeGreaterThanOrEqual(MIN_TEXT)
    }
  })

  it('reads at 4.5:1 on the default’s fill and hover as colors.css writes them', () => {
    const css = block(':root')
    for (const ground of [css['--accent']!, css['--accent-hover']!])
      expect(contrast(css['--on-accent']!, ground)).toBeGreaterThanOrEqual(MIN_TEXT)
  })
})

describe('accent text', () => {
  it('reads at 4.5:1 on every surface of every theme, a selected chip included', () => {
    for (const swatch of ACCENT_SWATCHES)
      for (const theme of THEMES) {
        const text = accentText(swatch, theme)
        const s = SURFACES[theme]
        for (const ground of [...s.list, over(swatch, s.soft, s.raised)])
          expect(contrast(text, ground), `${swatch} ${theme}`).toBeGreaterThanOrEqual(MIN_TEXT)
      }
  })

  it('is darkened for amber on the light theme and kept on the dark ones', () => {
    expect(accentText(AMBER, 'dark')).toBe(AMBER)
    expect(accentText(AMBER, 'black')).toBe(AMBER)
    const light = accentText(AMBER, 'light')
    expect(light).not.toBe(AMBER)
    expect(contrast(light, '#ffffff')).toBeGreaterThanOrEqual(MIN_TEXT)
  })
})

describe('accentTokens', () => {
  it('overrides every accent token for another swatch, in every theme', () => {
    const tokens = accentTokens(VIOLET)
    expect(tokens).not.toBeNull()
    for (const theme of THEMES) {
      expect(tokens![theme]['--violet']).toBe(VIOLET)
      expect(tokens![theme]['--on-accent']).toBe('#ffffff')
      expect(tokens![theme]['--focus-ring']).toBe(tokens![theme]['--accent-text'])
    }
    expect(tokens!.light['--accent-soft']).toBe(`${VIOLET}1f`)
    expect(tokens!.dark['--accent-soft']).toBe(`${VIOLET}2e`)
  })

  it('ignores a value that is not a colour', () => {
    expect(accentTokens('purple')).toBeNull()
  })
})
