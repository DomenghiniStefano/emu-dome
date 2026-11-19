import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const InputVariant = {
  // A text field of its own (SettingRow `.textfield`): 200 x 32, an edge on surface-raised.
  Field: 'field',
  // The same box as wide as its container, the placeholder muted, faded when disabled
  // (DetailsFiles «Nome mostrato»).
  Block: 'block',
  // The bare text inside a box drawn by its container (the search box of Settings, with its
  // icon): no edge, no fill, no outline, taking the room left. The browser's own small padding
  // stays, as it does today.
  Bare: 'bare',
} as const
export type InputVariant = (typeof InputVariant)[keyof typeof InputVariant]

export const inputVariants = cva('m-0 font-sans text-body text-ink disabled:cursor-not-allowed', {
  variants: {
    variant: {
      [InputVariant.Field]:
        'h-control w-field rounded-control border border-line-strong bg-surface-raised px-field-pad py-0',
      [InputVariant.Block]:
        'h-control rounded-control border border-line-strong bg-surface-raised px-field-pad py-0 placeholder:text-ink-muted disabled:opacity-disabled',
      [InputVariant.Bare]: 'min-w-0 flex-1 border-0 bg-transparent outline-none',
    },
  },
  defaultVariants: { variant: InputVariant.Field },
})
export type InputVariants = VariantProps<typeof inputVariants>
