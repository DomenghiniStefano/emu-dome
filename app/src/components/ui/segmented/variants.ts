import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const SegmentedVariant = {
  // One line per segment, wrapping when the choices do not fit (SettingRow `.segments`).
  Default: 'default',
  // The same, every segment semibold (ControlsRemap: «Si applica a», «Disposizione A/B»).
  Strong: 'strong',
  // Equal segments on one row, each two lines high, a name over its value (quick menu speeds,
  // G07). Its 52 height, 8 / 10 radii and 17 line are pending tokens (Cowork, spec
  // Decision 5). On the phone the radii are 10 / 12.
  Stacked: 'stacked',
} as const
export type SegmentedVariant = (typeof SegmentedVariant)[keyof typeof SegmentedVariant]

export const segmentedVariants = cva('', {
  variants: {
    variant: {
      [SegmentedVariant.Default]:
        'flex flex-wrap gap-2 rounded-control border border-line bg-surface-raised p-2',
      [SegmentedVariant.Strong]:
        'flex flex-wrap gap-2 rounded-control border border-line bg-surface-raised p-2',
      [SegmentedVariant.Stacked]:
        'grid auto-cols-fr grid-flow-col gap-2 rounded-card border border-line bg-surface-raised p-2 touch:rounded-12',
    },
  },
  defaultVariants: { variant: SegmentedVariant.Default },
})

export const segmentedItemVariants = cva('', {
  variants: {
    variant: {
      [SegmentedVariant.Default]:
        'h-control-sm cursor-pointer rounded-badge border-0 bg-transparent px-10 font-sans text-label text-ink-muted disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:font-semibold data-[state=on]:text-accent-text',
      [SegmentedVariant.Strong]:
        'h-control-sm cursor-pointer rounded-badge border-0 bg-transparent px-10 font-sans font-semibold text-label text-ink-muted disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:font-semibold data-[state=on]:text-accent-text',
      [SegmentedVariant.Stacked]:
        'flex h-52 min-w-0 cursor-pointer flex-col items-center justify-center rounded-8 border-0 bg-transparent leading-stacked disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:text-accent-text touch:rounded-card',
    },
  },
  defaultVariants: { variant: SegmentedVariant.Default },
})
export type SegmentedVariants = VariantProps<typeof segmentedVariants>
