import { cva } from 'class-variance-authority'

export const TabsSize = {
  // A section's tabs (ControlsView: 40px).
  Default: 'default',
  // The game details' tabs (GameDetails: 44px).
  Large: 'large',
} as const
export type TabsSize = (typeof TabsSize)[keyof typeof TabsSize]

// The active tab is ink with an accent-text underline drawn inside (`shadow-tab`), the others
// ink-muted until hovered (GameDetails, ControlsView `.tab`).
export const tabsTriggerVariants = cva(
  'shrink-0 cursor-pointer border-0 bg-transparent px-12 font-sans text-label whitespace-nowrap text-ink-muted hover:text-ink data-[state=active]:text-ink data-[state=active]:shadow-tab disabled:cursor-not-allowed',
  {
    variants: {
      size: {
        [TabsSize.Default]: 'h-tab',
        [TabsSize.Large]: 'h-tab-lg',
      },
    },
    defaultVariants: { size: TabsSize.Default },
  },
)
