import { cva } from 'class-variance-authority'

// A tooltip (Trello #104): the name of an icon alone, a small raised label with a thin edge, over
// everything (`z-tooltip`), never in the way of a press. In TV mode it is portaled outside the
// zoomed screens, so it takes the TV label's type itself.
export const tooltipVariants = cva(
  'pointer-events-none z-tooltip rounded-control border border-line bg-surface-raised px-6 py-2 text-caption text-ink shadow-card select-none tv:px-12 tv:py-6 tv:text-tv-label',
)

/** How far a tooltip sits from its element: the `--spacing-4` token, in CSS px. */
export const TOOLTIP_GAP = '--spacing-4'
/** How close to a window edge it may come: the `--spacing-8` token. */
export const TOOLTIP_EDGE = '--spacing-8'
