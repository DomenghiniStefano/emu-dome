// The touch controls' two styles (C08, `touch.tskin`, owner 2026-11-22), drawn from the design
// system's tokens: «Fumé» (`smoke`, dark see-through glass, the default) and «Contorno»
// (`outline`, the outline alone). Both are opaque colours; each whole control is faded by one
// opacity (`opacity-pad`, TouchControl), so the game shows through and an edge never darkens at
// a joint. Settings files of an older layout are moved to these ids by the settings engine
// (store.rs, layout 2). Imported skins (our pack format, `.deltaskin`, RetroArch overlays) are
// Phase 5 (docs/decisions.md).
import { ButtonVariant } from '@/components/ui/button'

export const TOUCH_SKINS = ['smoke', 'outline'] as const
export type TouchSkin = (typeof TOUCH_SKINS)[number]

/** A `touch.tskin` value: anything unknown is «Fumé». */
export const touchSkin = (value: unknown): TouchSkin =>
  TOUCH_SKINS.find(s => s === value) ?? 'smoke'

/** The buttons' look per style (they colour the shape drawn inside them, PadShape.vue). */
export const SKIN_BUTTON: Record<TouchSkin, ButtonVariant> = {
  smoke: ButtonVariant.PadSmoke,
  outline: ButtonVariant.PadOutline,
}

/** The d-pad's and the Circle Pad's colours per style, the same as the buttons': `base` on their
 * drawing (fill and edge inherited by its shapes, the arrows in the letters' colour), `held` on a
 * lit arm or the moved knob. */
export const SKIN_PART: Record<TouchSkin, { base: string; held: string }> = {
  smoke: {
    base: 'fill-pad-smoke stroke-pad-smoke-edge text-pad-ink',
    held: 'fill-pad-smoke-held',
  },
  outline: {
    base: 'fill-none stroke-pad-outline-edge text-pad-ink',
    held: 'fill-pad-outline-held',
  },
}
