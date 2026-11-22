import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const PopoverVariant = {
  // A panel of choices under its button (L03 «Ordina e filtra»): thick glass, the menu's width.
  Panel: 'panel',
  // The notifications under the top bar's bell (X03): `notifications` wide (the window's width
  // less its margins on the phone), its list scrolling when the window is short.
  Notifications: 'notifications',
} as const
export type PopoverVariant = (typeof PopoverVariant)[keyof typeof PopoverVariant]

export const popoverVariants = cva('glass-thick z-raised flex flex-col', {
  variants: {
    variant: {
      [PopoverVariant.Panel]: 'w-menu gap-14 p-16',
      [PopoverVariant.Notifications]:
        'max-h-[calc(100dvh-var(--spacing-titlebar)-var(--spacing-32))] w-notifications max-w-[calc(100vw-var(--spacing-32))] gap-4 p-12',
    },
  },
  defaultVariants: { variant: PopoverVariant.Panel },
})
export type PopoverVariants = VariantProps<typeof popoverVariants>

/** How far a popover sits from its button: the `--spacing-8` token, in CSS px. */
export const POPOVER_GAP = '--spacing-8'
