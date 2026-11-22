// The built-in skins of the touch controls (C08, `touch.tskin`), drawn from the design system's
// tokens: «Vetro» (the glass pad), «Solo contorni» (an ink edge, no fill) and «Contrasto alto»
// (the ground's solid fill and an ink edge). Imported skins (our pack format, `.deltaskin`,
// RetroArch overlays) are Phase 5 (docs/decisions.md).
import { ButtonVariant } from '@/components/ui/button'

export const TOUCH_SKINS = ['glass', 'outlines', 'contrast'] as const
export type TouchSkin = (typeof TOUCH_SKINS)[number]

/** A `touch.tskin` value: anything unknown is «Vetro». */
export const touchSkin = (value: unknown): TouchSkin =>
  TOUCH_SKINS.find(s => s === value) ?? 'glass'

/** The buttons' look per skin. */
export const SKIN_BUTTON: Record<TouchSkin, ButtonVariant> = {
  glass: ButtonVariant.Pad,
  outlines: ButtonVariant.PadOutline,
  contrast: ButtonVariant.PadContrast,
}

/** The d-pad's arms and the stick's ring and knob per skin: at rest and while pressed. */
export const SKIN_PART: Record<TouchSkin, { rest: string; held: string }> = {
  glass: {
    rest: 'border border-line-strong bg-glass',
    held: 'border border-line-strong bg-surface-hover',
  },
  outlines: {
    rest: 'border-strong border-ink bg-transparent',
    held: 'border-strong border-ink bg-glass',
  },
  contrast: {
    rest: 'border-strong border-ink bg-bg',
    held: 'border-strong border-ink bg-ink',
  },
}
