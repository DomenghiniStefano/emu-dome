import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const PopoverVariant = {
  // A panel of choices under its button (L03 «Ordina e filtra»): thick glass, the menu's width.
  Panel: 'panel',
} as const
export type PopoverVariant = (typeof PopoverVariant)[keyof typeof PopoverVariant]

export const popoverVariants = cva('glass-thick z-raised flex flex-col', {
  variants: {
    variant: {
      [PopoverVariant.Panel]: 'w-menu gap-14 p-16',
    },
  },
  defaultVariants: { variant: PopoverVariant.Panel },
})
export type PopoverVariants = VariantProps<typeof popoverVariants>

/** How far a popover sits from its button: the `--spacing-8` token, in CSS px. */
export const POPOVER_GAP = '--spacing-8'
