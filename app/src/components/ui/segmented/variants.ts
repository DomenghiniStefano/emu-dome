import type { InjectionKey, Ref } from 'vue'
import { cva } from 'class-variance-authority'

export const SegmentedVariant = {
  // A setting's choice (SettingRow `.segments`): the chosen segment semibold, the others medium.
  Default: 'default',
  // Every segment semibold (ControlsRemap: «Si applica a», «Disposizione A/B»).
  Strong: 'strong',
} as const
export type SegmentedVariant = (typeof SegmentedVariant)[keyof typeof SegmentedVariant]

/** The variant Segmented gives its items. */
export const SegmentedVariantKey: InjectionKey<Ref<SegmentedVariant>> = Symbol('SegmentedVariant')

export const segmentedItemVariants = cva(
  'h-control-sm cursor-pointer rounded-4 border-0 bg-transparent px-10 font-sans text-label text-ink-muted disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:font-semibold data-[state=on]:text-accent-text',
  {
    variants: {
      variant: {
        [SegmentedVariant.Default]: '',
        [SegmentedVariant.Strong]: 'font-semibold',
      },
    },
    defaultVariants: { variant: SegmentedVariant.Default },
  },
)
