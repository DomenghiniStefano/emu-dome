import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

// A short status beside a name. The variant is its colour (what it says), the size its shape.
export const BadgeVariant = {
  // A fact («Più recente», «Solo su questo computer»): raised fill, muted ink. Never the accent:
  // a badge does not choose for the user.
  Neutral: 'neutral',
  // A problem («Salvataggio illeggibile»).
  Warning: 'warning',
  // Something not here yet or reserved («In arrivo», «Con Nintendo DS»): a dashed edge, no fill.
  Reserved: 'reserved',
  // A good state («In uso» on the controller in use).
  Success: 'success',
  // A value shown quietly beside a control (a setting's chosen file, «Nessun file scelto»).
  Quiet: 'quiet',
} as const
export type BadgeVariant = (typeof BadgeVariant)[keyof typeof BadgeVariant]

export const BadgeSize = {
  // A tag: square corners (`radius-badge`), 11px on a 16px line (`badge`, pending Cowork).
  Tag: 'tag',
  // A pill: round, the caption on the body's line (`caption-tall`).
  Pill: 'pill',
} as const
export type BadgeSize = (typeof BadgeSize)[keyof typeof BadgeSize]

export const badgeVariants = cva('flex-none whitespace-nowrap', {
  variants: {
    variant: {
      [BadgeVariant.Neutral]: 'bg-surface-raised font-medium text-ink-muted',
      [BadgeVariant.Warning]: 'bg-warning-soft text-warning',
      [BadgeVariant.Reserved]: 'border border-dashed border-line-strong font-medium text-ink-muted',
      [BadgeVariant.Success]: 'bg-success-soft font-semibold text-success',
      [BadgeVariant.Quiet]: 'bg-surface-hover text-ink-muted',
    },
    size: {
      [BadgeSize.Tag]: 'rounded-badge px-6 text-badge',
      [BadgeSize.Pill]: 'rounded-full px-8 py-2 text-caption-tall',
    },
  },
  defaultVariants: { variant: BadgeVariant.Neutral, size: BadgeSize.Tag },
})
export type BadgeVariants = VariantProps<typeof badgeVariants>
