// The touch controls' two styles (C08, `touch.tskin`, owner 2026-11-22), drawn from the design
// system's tokens: «Fumé» (`smoke`, dark see-through glass, the default) and «Contorno»
// (`outline`, the outline alone). Settings files of an older layout are moved to these ids by
// the settings engine (store.rs, layout 2). Imported skins (our pack format, `.deltaskin`,
// RetroArch overlays) are Phase 5 (docs/decisions.md).
import { ButtonVariant } from '@/components/ui/button'

export const TOUCH_SKINS = ['smoke', 'outline'] as const
export type TouchSkin = (typeof TOUCH_SKINS)[number]

/** A `touch.tskin` value: anything unknown is «Fumé». */
export const touchSkin = (value: unknown): TouchSkin =>
  TOUCH_SKINS.find(s => s === value) ?? 'smoke'

/** The buttons' look per style. */
export const SKIN_BUTTON: Record<TouchSkin, ButtonVariant> = {
  smoke: ButtonVariant.Pad,
  outline: ButtonVariant.PadOutline,
}

/** The d-pad's arms and the stick's ring and knob per style: at rest and while pressed. */
export const SKIN_PART: Record<TouchSkin, { rest: string; held: string }> = {
  smoke: {
    rest: 'border border-line-strong bg-glass',
    held: 'border border-line-strong bg-surface-hover',
  },
  outline: {
    rest: 'border-strong border-ink bg-transparent',
    held: 'border-strong border-ink bg-glass',
  },
}
