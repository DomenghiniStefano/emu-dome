import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

// Where an empty state sits. Every one has the same parts in the same order: an icon tile, a
// title, a line of text, the actions, a footer line (each optional).
export const EmptyStateVariant = {
  // Fills the section it is the whole of (E03 no saves, E05 no controller), centred.
  Page: 'page',
  // A dashed box that invites a drop as well as a click (L02, the empty library).
  Framed: 'framed',
  // A card inside a list that has nothing to show (the settings' search with no results).
  Card: 'card',
} as const
export type EmptyStateVariant = (typeof EmptyStateVariant)[keyof typeof EmptyStateVariant]

export const emptyStateVariants = cva(
  'flex flex-col items-center justify-center gap-8 text-center',
  {
    variants: {
      variant: {
        [EmptyStateVariant.Page]: 'flex-1 p-16',
        [EmptyStateVariant.Framed]:
          'max-w-empty rounded-panel border border-dashed border-line-strong p-24',
        [EmptyStateVariant.Card]: 'rounded-card border border-line bg-surface p-card-pad',
      },
    },
    defaultVariants: { variant: EmptyStateVariant.Page },
  },
)
export type EmptyStateVariants = VariantProps<typeof emptyStateVariants>
