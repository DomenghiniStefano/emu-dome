import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const SegmentedVariant = {
  // One line per segment, wrapping when the choices do not fit (SettingRow `.segments`).
  Default: 'default',
  // Equal segments on one row, each two lines high, a name over its value (quick menu speeds,
  // G07). Its 52 height, 8 / 12 radii and 17 line are off the design system's scales: kept until
  // Cowork decides (spec Decision 5). On the phone the radii grow to 10 / 12.
  Stacked: 'stacked',
  StackedTouch: 'stackedTouch',
} as const
export type SegmentedVariant = (typeof SegmentedVariant)[keyof typeof SegmentedVariant]

export const segmentedVariants = cva('', {
  variants: {
    variant: {
      [SegmentedVariant.Default]:
        'flex flex-wrap gap-2 rounded-6 border border-line bg-surface-raised p-2',
      [SegmentedVariant.Stacked]:
        'grid auto-cols-fr grid-flow-col gap-2 rounded-10 border border-line bg-surface-raised p-2',
      [SegmentedVariant.StackedTouch]:
        'grid auto-cols-fr grid-flow-col gap-2 rounded-[12px] border border-line bg-surface-raised p-2',
    },
  },
  defaultVariants: { variant: SegmentedVariant.Default },
})

export const segmentedItemVariants = cva('', {
  variants: {
    variant: {
      [SegmentedVariant.Default]:
        'h-control-sm cursor-pointer rounded-4 border-0 bg-transparent px-10 font-sans text-label text-ink-muted disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:font-semibold data-[state=on]:text-accent-text',
      [SegmentedVariant.Stacked]:
        'flex h-[52px] min-w-0 cursor-pointer flex-col items-center justify-center rounded-[8px] border-0 bg-transparent leading-[17px] disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:text-accent-text',
      [SegmentedVariant.StackedTouch]:
        'flex h-[52px] min-w-0 cursor-pointer flex-col items-center justify-center rounded-10 border-0 bg-transparent leading-[17px] disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:text-accent-text',
    },
  },
  defaultVariants: { variant: SegmentedVariant.Default },
})
export type SegmentedVariants = VariantProps<typeof segmentedVariants>
