import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

// A key of the keyboard or a button of the controller, drawn as a key: an edge, no fill.
export const KbdVariant = {
  // On a surface: a strong edge, ink.
  Default: 'default',
  // On the main action's accent fill (the quick menu's «Riprendi il gioco»).
  OnAccent: 'onAccent',
} as const
export type KbdVariant = (typeof KbdVariant)[keyof typeof KbdVariant]

export const KbdSize = {
  // In a line of text or a list (G06 «Esc o Home per il menu rapido», C01's shortcuts): 20 tall,
  // the caption on the body's line.
  Default: 'default',
  // A hint beside an action's label (Esc, B): 18 tall, the badge's type (pending Cowork).
  Small: 'small',
  // TV mode (#30): a face button of the connected controller, seen from the sofa (the button bar,
  // the hero's actions): a round key, 30 tall, the TV hint's type.
  TvFace: 'tvFace',
  // TV mode: a shoulder button (L1 R1, LB RB): the same key with a control's corners.
  TvShoulder: 'tvShoulder',
} as const
export type KbdSize = (typeof KbdSize)[keyof typeof KbdSize]

// The corners are the size's: a round TV key would lose to a shared radius.
export const kbdVariants = cva(
  'inline-flex items-center border font-sans font-semibold whitespace-nowrap',
  {
    variants: {
      variant: {
        [KbdVariant.Default]: 'border-line-strong text-ink',
        [KbdVariant.OnAccent]: 'border-on-accent-edge text-on-accent',
      },
      size: {
        [KbdSize.Default]: 'h-20 rounded-badge px-6 text-caption-tall',
        [KbdSize.Small]: 'h-18 rounded-badge px-5 text-badge',
        [KbdSize.TvFace]:
          'h-tv-glyph min-w-tv-glyph justify-center rounded-full px-8 text-tv-hint font-semibold',
        [KbdSize.TvShoulder]:
          'h-tv-glyph min-w-tv-glyph justify-center rounded-control px-8 text-tv-hint font-semibold',
      },
    },
    defaultVariants: { variant: KbdVariant.Default, size: KbdSize.Default },
  },
)
export type KbdVariants = VariantProps<typeof kbdVariants>
