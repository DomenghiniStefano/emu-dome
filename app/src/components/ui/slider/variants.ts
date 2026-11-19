import { cva } from 'class-variance-authority'

export const SliderVariant = {
  // A 4px track edged in `line`, filled inside its edge with a rounded fill.
  Default: 'default',
  // The settings rows' slider (SettingRow, the native range it replaced): the track itself is
  // painted, edge included, with a hard stop at the value (`--fill`, set by Slider.vue), as the
  // range's track was: square where the fill stops, the edge line over it. The theme's runtime
  // colours (`--accent-text`, `--surface-raised`): the `--color-*` tokens are inlined, not variables.
  EdgeFill: 'edgeFill',
} as const
export type SliderVariant = (typeof SliderVariant)[keyof typeof SliderVariant]

export const sliderTrackVariants = cva(
  'relative h-4 flex-1 rounded-full border border-line bg-surface-raised',
  {
    variants: {
      variant: {
        [SliderVariant.Default]: '',
        [SliderVariant.EdgeFill]:
          'bg-[linear-gradient(to_right,var(--accent-text)_var(--fill),var(--surface-raised)_var(--fill))]',
      },
    },
    defaultVariants: { variant: SliderVariant.Default },
  },
)

/** The fill inside the track (the Default variant; EdgeFill paints the track instead). */
export const sliderRangeClass = 'absolute h-full rounded-full bg-accent-text'

/** How far along the range `value` is, in percent (0 when the range is empty). */
export function fillPercent(value: number, min: number, max: number): number {
  if (max <= min) return 0
  return ((Math.min(Math.max(value, min), max) - min) / (max - min)) * 100
}
