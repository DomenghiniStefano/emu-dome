import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const PopoverVariant = {
  // A panel of choices under its button (L03 «Ordina e filtra»): thick glass, the menu's width,
  // never taller than the room Reka leaves under the button: its body scrolls, its footer (the
  // `footer` slot of PopoverContent) stays in view under it (#106).
  Panel: 'panel',
  // The notifications under the top bar's bell (X03): `notifications` wide (the window's width
  // less its margins on the phone), its list scrolling when the window is short.
  Notifications: 'notifications',
  // As wide as what it holds (a date field's segments, #102): thick glass, a small padding.
  Fit: 'fit',
} as const
export type PopoverVariant = (typeof PopoverVariant)[keyof typeof PopoverVariant]

export const popoverVariants = cva('glass-thick z-raised flex flex-col', {
  variants: {
    variant: {
      [PopoverVariant.Panel]: 'max-h-(--reka-popover-content-available-height) w-menu gap-8 p-10',
      [PopoverVariant.Notifications]:
        'max-h-[calc(100dvh-var(--spacing-titlebar)-var(--spacing-32))] w-notifications max-w-[calc(100vw-var(--spacing-32))] gap-2 p-6',
      [PopoverVariant.Fit]: 'w-fit gap-6 p-8',
    },
  },
  defaultVariants: { variant: PopoverVariant.Panel },
})
export type PopoverVariants = VariantProps<typeof popoverVariants>

/**
 * A Panel's parts (#106). The body scrolls when the window is short: it reaches the panel's sides
 * (the rows' `-mx-6` and the focus ring stay inside it, not cut by its scrolling) and gives the
 * ring its room above and below. The footer never scrolls away.
 */
export const POPOVER_PANEL_BODY =
  '-mx-10 -my-4 flex min-h-0 flex-col gap-8 overflow-y-auto overscroll-contain px-10 py-4'
export const POPOVER_PANEL_FOOTER = 'flex flex-none flex-col gap-8'

/** How far a popover sits from its button: the `--spacing-8` token, in CSS px. */
export const POPOVER_GAP = '--spacing-8'
