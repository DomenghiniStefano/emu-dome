import { describe, expect, it } from 'vitest'
import colors from '@/assets/theme/colors.css?raw'
import motion from '@/assets/theme/motion.css?raw'
import radius from '@/assets/theme/radius.css?raw'
import spacing from '@/assets/theme/spacing.css?raw'
import typography from '@/assets/theme/typography.css?raw'
import { ThemeNamespace, themeKeys, utilityKeys } from './themeKeys'

describe('themeKeys', () => {
  it('returns every name declared in the namespace, once', () => {
    const css = [
      '@theme {',
      '  --text-body: 13px;',
      '  --text-body--line-height: 18px;',
      '  --text-caption: 12px;',
      '}',
    ].join('\n')
    expect(themeKeys(css, ThemeNamespace.Text)).toEqual(['body', 'caption'])
  })

  it('skips the namespace reset', () => {
    const css = '--radius-*: initial;\n--radius-6: 6px;'
    expect(themeKeys(css, ThemeNamespace.Radius)).toEqual(['6'])
  })

  it('keeps a multi-word name whole', () => {
    expect(themeKeys('--spacing-control-sm: 28px;', ThemeNamespace.Spacing)).toEqual(['control-sm'])
  })

  it('reads a hyphenated namespace', () => {
    const css = '--transition-duration-fast: 150ms;'
    expect(themeKeys(css, ThemeNamespace.TransitionDuration)).toEqual(['fast'])
  })

  it('keeps a longer namespace out of a shorter one', () => {
    const css = '--font-sans: x;\n--font-weight-medium: 500;'
    expect(themeKeys(css, ThemeNamespace.Font)).toEqual(['sans'])
    expect(themeKeys(css, ThemeNamespace.FontWeight)).toEqual(['medium'])
  })

  it('ignores references, other namespaces and look-alike variables', () => {
    const css = [
      '--color-text-muted: #ffffff;',
      '--default-transition-duration: 150ms;',
      '--default-transition-timing-function: ease;',
      'font-size: var(--text-body);',
      '--duration-ambient: var(--transition-duration-ambient);',
    ].join('\n')
    expect(themeKeys(css, ThemeNamespace.Text)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.TransitionDuration)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.Ease)).toEqual([])
  })

  it('reads the real theme files as source, not as compiled CSS', () => {
    expect(typography).toContain('@theme')
    expect(themeKeys(typography, ThemeNamespace.Text)).toEqual([
      'stat',
      'display',
      'lead',
      'title',
      'heading',
      'body',
      'label',
      'caption',
      'overline',
      'cta',
      'dialog',
      'caption-tall',
      'caption-tight',
      'label-tight',
      'fine',
      'badge',
      'overline-lg',
      'label-lead',
      'hero',
      'display-md',
      'display-sm',
      'page-title',
      'stat-md',
      'rail',
      'wordmark',
      'console-badge',
      'overline-inline',
      'caption-inline',
      'label-inline',
      'cta-inline',
    ])
    expect(themeKeys(radius, ThemeNamespace.Radius)).toEqual([
      '4',
      '6',
      '10',
      '16',
      '8',
      '12',
      '20',
      'badge',
      'control',
      'card',
      'panel',
    ])
    expect(themeKeys(motion, ThemeNamespace.TransitionDuration)).toEqual([
      'fast',
      'panel',
      'scene',
      'ambient',
      'hint',
    ])
    expect(themeKeys(spacing, ThemeNamespace.Spacing)).toContain('control')
    expect(themeKeys(colors, ThemeNamespace.Color)).toContain('accent-soft')
  })
})

describe('utilityKeys', () => {
  it('reads the names of @utility rules with a prefix', () => {
    const css = '@utility z-menu {\n  z-index: 20;\n}\n@utility glass-thick {\n}'
    expect(utilityKeys(css, 'z')).toEqual(['menu'])
  })
})
