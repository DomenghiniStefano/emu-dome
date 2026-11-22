import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const DateTimeFieldVariant = {
  // The field of a setting row (SettingRow, `saves.rtcdate`): the Select field's box (200px at
  // least, 32px high, an edge on surface-raised), the date and time in it, a calendar icon at the
  // end; the segments open in a popover under it.
  Field: 'field',
} as const
export type DateTimeFieldVariant = (typeof DateTimeFieldVariant)[keyof typeof DateTimeFieldVariant]

export const dateTimeFieldVariants = cva(
  'flex cursor-pointer items-center gap-8 rounded-control border px-field-pad text-left font-sans text-label enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-disabled',
  {
    variants: {
      variant: {
        [DateTimeFieldVariant.Field]:
          'h-control min-w-field border-line bg-surface-raised text-inherit tabular-nums',
      },
    },
    defaultVariants: { variant: DateTimeFieldVariant.Field },
  },
)
export type DateTimeFieldVariants = VariantProps<typeof dateTimeFieldVariants>

/** One segment of the open field (day, month, year, hour, minute; a literal between them). */
export const dateSegmentVariants = cva('font-sans text-lead tabular-nums', {
  variants: {
    literal: {
      true: 'text-ink-muted',
      false: 'rounded-control px-4 py-2 text-ink outline-none focus:bg-accent-soft',
    },
  },
  defaultVariants: { literal: false },
})
