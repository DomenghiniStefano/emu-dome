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
  // One of a choice of pictures (CoverPicker): opaque surface, `selected` on accent-soft with an
  // accent-text edge; `data-empty` (nothing to show yet, «add one») draws a dashed strong edge.
  Choice: 'choice',
  // One row of a list the user picks from, rendered `as="button"` (Saves: the games): cover,
  // text and badge in a row, no fill until hovered, `selected` on accent-soft. Its 8px radius is
  // off the scale, kept until Cowork decides (spec Decision 5: 8 → 6).
  ListItem: 'listItem',
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
        'rounded-10 border border-line bg-surface p-10 data-empty:not-data-selected:border-dashed data-empty:not-data-selected:border-line-strong data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.ListItem]:
        'flex-row items-center gap-10 rounded-[8px] border-0 bg-transparent p-8 hover:not-data-selected:bg-surface-hover data-selected:bg-accent-soft',
    },
  },
  defaultVariants: { variant: CardVariant.Surface },
})
export type CardVariants = VariantProps<typeof cardVariants>
