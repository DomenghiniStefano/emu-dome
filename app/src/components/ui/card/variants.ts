import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const CardVariant = {
  // An opaque card on a page (Details: `.card` in DetailsFiles, DetailsControls).
  Surface: 'surface',
  // A card inside a card, or one of a choice of cards (SettingRow `.card`); `selected` marks the
  // chosen one.
  Raised: 'raised',
  // A glass panel over the ambient haze (README «Vetro»: thick glass for menus and panels).
  Glass: 'glass',
  // One of a choice of cards on a page (first run S04, A/B layout): Surface with 14 inside;
  // `selected` marks the chosen one as Raised does.
  Choice: 'choice',
} as const
export type CardVariant = (typeof CardVariant)[keyof typeof CardVariant]

// The gap between the card's parts is the caller's: the screens use 6, 10 and 14 today.
export const cardVariants = cva('flex flex-col text-left', {
  variants: {
    variant: {
      [CardVariant.Surface]: 'rounded-10 border border-line bg-surface p-16',
      [CardVariant.Raised]:
        'rounded-10 border border-line bg-surface-raised p-10 data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.Glass]: 'glass-thick p-24',
      [CardVariant.Choice]:
        'rounded-10 border border-line bg-surface p-14 data-selected:border-accent-text data-selected:bg-accent-soft',
    },
  },
  defaultVariants: { variant: CardVariant.Surface },
})
export type CardVariants = VariantProps<typeof cardVariants>
