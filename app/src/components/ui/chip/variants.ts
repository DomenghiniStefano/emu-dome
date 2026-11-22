import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const ChipVariant = {
  // A filter of a list (LibraryView `.chip`): a control high, a chosen one on accent-soft with a soft
  // accent edge.
  Filter: 'filter',
  // An option of a multiple choice (SettingRow `.chip`): control-sm, a chosen one edged in accent-text.
  Option: 'option',
} as const
export type ChipVariant = (typeof ChipVariant)[keyof typeof ChipVariant]

// A disabled chip looks like the others: today's screens fade the whole row that holds it
// (SettingRow `.locked`), not the chip.
// `data-selected` is set by Chip from its `selected` prop; aria-pressed / aria-checked stay the
// caller's, who knows which role the chip has.
export const chipVariants = cva(
  'inline-flex shrink-0 items-center gap-4 rounded-control border border-line bg-surface-raised font-sans text-label text-ink whitespace-nowrap enabled:cursor-pointer disabled:cursor-not-allowed data-selected:bg-accent-soft data-selected:text-accent-text',
  {
    variants: {
      variant: {
        [ChipVariant.Filter]:
          'h-control px-8 enabled:hover:not-data-selected:bg-surface-hover data-selected:border-accent-edge',
        [ChipVariant.Option]: 'h-control-sm px-8 data-selected:border-accent-text',
      },
    },
    defaultVariants: { variant: ChipVariant.Filter },
  },
)
export type ChipVariants = VariantProps<typeof chipVariants>
