import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const DropdownMenuVariant = {
  // A game's «…» (L01 on a cover, L04): thick glass, the menu's own least width.
  Actions: 'actions',
} as const
export type DropdownMenuVariant = (typeof DropdownMenuVariant)[keyof typeof DropdownMenuVariant]

export const dropdownMenuVariants = cva('glass-thick z-raised flex flex-col', {
  variants: {
    variant: {
      [DropdownMenuVariant.Actions]: 'min-w-menu-min gap-px p-2',
    },
  },
  defaultVariants: { variant: DropdownMenuVariant.Actions },
})
export type DropdownMenuVariants = VariantProps<typeof dropdownMenuVariants>

/** How far a menu sits from its button: the `--spacing-6` token, in CSS px. */
export const DROPDOWN_MENU_GAP = '--spacing-6'
