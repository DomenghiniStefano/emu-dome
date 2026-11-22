import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const CaptureFieldVariant = {
  // A remap row's button on the device (ControlsRemap): a raised field naming it; listening, a
  // dashed accent edge on no fill. A control high, as Select and Input beside it.
  Field: 'field',
  // A shortcut row's «Cambia» (ControlsShortcuts): quiet text, ringed with a dashed accent line
  // while it listens.
  Quiet: 'quiet',
} as const
export type CaptureFieldVariant = (typeof CaptureFieldVariant)[keyof typeof CaptureFieldVariant]

// `data-listening` is set by CaptureField from its `listening` prop. The Quiet ring is an
// outline, so the global focus rules (styles.css) replace it while the field has the focus, as
// they did over ControlsShortcuts' own `.ghost.listening`.
export const captureFieldVariants = cva(
  'cursor-pointer rounded-control py-0 font-sans whitespace-nowrap select-none',
  {
    variants: {
      variant: {
        [CaptureFieldVariant.Field]:
          'h-control border border-line bg-surface-raised px-8 text-label text-ink hover:not-data-listening:bg-surface-hover data-listening:border-dashed data-listening:border-accent-text data-listening:bg-transparent data-listening:text-accent-text',
        [CaptureFieldVariant.Quiet]:
          'h-control-sm border-0 bg-transparent px-6 text-body text-ink-muted hover:bg-surface-hover data-listening:text-accent-text data-listening:outline data-listening:outline-accent-text data-listening:outline-dashed',
      },
    },
    defaultVariants: { variant: CaptureFieldVariant.Field },
  },
)
export type CaptureFieldVariants = VariantProps<typeof captureFieldVariants>
