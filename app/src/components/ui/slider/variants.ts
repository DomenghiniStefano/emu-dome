import { cva } from 'class-variance-authority'

export const SliderVariant = {
  // A 4px track edged in `line`, filled inside its edge with a rounded fill.
  Default: 'default',
  // The settings rows' slider (SettingRow, the native range it replaced): the fill reaches the
  // track's own edge, square where it stops, with the edge line drawn over it.
  EdgeFill: 'edgeFill',
} as const
export type SliderVariant = (typeof SliderVariant)[keyof typeof SliderVariant]

export const sliderTrackVariants = cva('relative h-4 flex-1 rounded-full bg-surface-raised', {
  variants: {
    variant: {
      [SliderVariant.Default]: 'border border-line',
      [SliderVariant.EdgeFill]:
        'overflow-hidden after:absolute after:inset-0 after:rounded-full after:border after:border-line',
    },
  },
  defaultVariants: { variant: SliderVariant.Default },
})

export const sliderRangeVariants = cva('absolute h-full bg-accent-text', {
  variants: {
    variant: {
      [SliderVariant.Default]: 'rounded-full',
      [SliderVariant.EdgeFill]: '',
    },
  },
  defaultVariants: { variant: SliderVariant.Default },
})
