import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const SegmentedVariant = {
  // One line per segment, wrapping when the choices do not fit (SettingRow `.segments`).
  Default: 'default',
  // The same, every segment semibold (ControlsRemap: «Si applica a», «Disposizione A/B»).
  Strong: 'strong',
  // Equal segments on one row, a name and its value (quick menu speeds, G07). Owner, 2026-11-22
  // («lo stacked è gigantesco»): the two sit on one line, a control high (the touch minimum on
  // the phone), no longer one over the other; the name keeps the variant's name.
  Stacked: 'stacked',
} as const
export type SegmentedVariant = (typeof SegmentedVariant)[keyof typeof SegmentedVariant]

export const segmentedVariants = cva('', {
  variants: {
    variant: {
      [SegmentedVariant.Default]:
        'flex flex-wrap gap-2 rounded-control border border-line bg-surface-raised p-px',
      [SegmentedVariant.Strong]:
        'flex flex-wrap gap-2 rounded-control border border-line bg-surface-raised p-px',
      [SegmentedVariant.Stacked]:
        'grid auto-cols-fr grid-flow-col gap-2 rounded-control border border-line bg-surface-raised p-px',
    },
  },
  defaultVariants: { variant: SegmentedVariant.Default },
})

export const segmentedItemVariants = cva('', {
  variants: {
    variant: {
      [SegmentedVariant.Default]:
        'h-control-sm cursor-pointer rounded-badge border-0 bg-transparent px-8 font-sans text-label text-ink-muted disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:font-semibold data-[state=on]:text-accent-text',
      [SegmentedVariant.Strong]:
        'h-control-sm cursor-pointer rounded-badge border-0 bg-transparent px-8 font-sans font-semibold text-label text-ink-muted disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:font-semibold data-[state=on]:text-accent-text',
      [SegmentedVariant.Stacked]:
        'flex h-control min-w-0 cursor-pointer flex-row items-baseline justify-center gap-4 rounded-badge border-0 bg-transparent px-8 disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:text-accent-text touch:h-touch touch:items-center',
    },
  },
  defaultVariants: { variant: SegmentedVariant.Default },
})
export type SegmentedVariants = VariantProps<typeof segmentedVariants>
