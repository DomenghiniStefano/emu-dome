// The looks of the Saves section's own small parts, as `as const` constants keyed into cva, the
// way the primitives do it (components/ui/*/variants.ts): never a string in a template.
import { cva } from 'class-variance-authority'

/** SaveBadge: neutral for a fact («Più recente»), warning for a problem («Salvataggio illeggibile»). */
export const SaveBadgeTone = {
  Neutral: 'neutral',
  Warning: 'warning',
} as const
export type SaveBadgeTone = (typeof SaveBadgeTone)[keyof typeof SaveBadgeTone]

// 11px on a 16px line is off the type scale (its 11px step sits on a 14px line): kept until Cowork places
// it (spec Decision 5).
export const saveBadgeVariants = cva('rounded-4 px-6 text-badge', {
  variants: {
    tone: {
      [SaveBadgeTone.Neutral]: 'bg-surface-raised font-medium text-ink-muted',
      [SaveBadgeTone.Warning]: 'flex-none whitespace-nowrap bg-warning-soft text-warning',
    },
  },
  defaultVariants: { tone: SaveBadgeTone.Neutral },
})
