import { cva } from 'class-variance-authority'

export const SliderVariant = {
  // A 4px track edged in `line`, filled inside its edge with a rounded fill.
  Default: 'default',
  // The settings rows' slider (SettingRow, the native range it replaced, whose fill was a
  // gradient tiled over the track's inner box): the fill covers the track's whole height, square
  // where it stops, under the edge line; the edge's first column stays the track's colour and its
  // last one the fill's, as the tiled gradient drew them.
  EdgeFill: 'edgeFill',
} as const
export type SliderVariant = (typeof SliderVariant)[keyof typeof SliderVariant]

export const sliderTrackVariants = cva('relative h-4 flex-1 rounded-full bg-surface-raised', {
  variants: {
    variant: {
      [SliderVariant.Default]: 'border border-line',
      [SliderVariant.EdgeFill]:
        'overflow-hidden before:absolute before:inset-y-0 before:right-0 before:w-px before:bg-accent-text after:absolute after:inset-0 after:rounded-full after:border after:border-line',
    },
  },
  defaultVariants: { variant: SliderVariant.Default },
})

export const sliderRangeVariants = cva('absolute h-full bg-accent-text', {
  variants: {
    variant: {
      [SliderVariant.Default]: 'rounded-full',
      [SliderVariant.EdgeFill]: 'ml-px',
    },
  },
  defaultVariants: { variant: SliderVariant.Default },
})
