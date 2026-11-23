import { cva } from 'class-variance-authority'

// A progress bar: a 6px track on raised surface, near-square corners, the done part in accent text. Its width is the
// caller's (a scan's `max-w-scan`, a scene's `w-progress`).
export const progressVariants = cva('block h-6 overflow-hidden rounded-badge bg-surface-raised')

// The band: the done part when the total is known, else a third of the track sweeping across it
// (`animate-progress-sweep`, utilities.css; still and full with «Riduci le animazioni»).
export const progressIndicatorVariants = cva('block h-full bg-accent-text', {
  variants: {
    indeterminate: {
      true: 'w-3/10 animate-progress-sweep',
      false: 'transition-[width] duration-panel ease-out',
    },
  },
  defaultVariants: { indeterminate: false },
})
