import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const DialogTone = {
  // A question or a notice: the title in ink.
  Default: 'default',
  // Something failed (G02 cannot start, E11 the emulator stopped).
  Danger: 'danger',
  // Something needs a look but nothing is lost (E04 the save cannot be read, E12 slow).
  Warning: 'warning',
} as const
export type DialogTone = (typeof DialogTone)[keyof typeof DialogTone]

export const dialogTitleVariants = cva('m-0 text-dialog', {
  variants: {
    tone: {
      [DialogTone.Default]: 'text-ink',
      [DialogTone.Danger]: 'text-danger',
      [DialogTone.Warning]: 'text-warning',
    },
  },
  defaultVariants: { tone: DialogTone.Default },
})
export type DialogTitleVariants = VariantProps<typeof dialogTitleVariants>
