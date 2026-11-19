import { cva } from 'class-variance-authority'

export const TabsSize = {
  // A section's tabs (ControlsView: 40px).
  Default: 'default',
  // The game details' tabs (GameDetails: 44px). On the phone pills that scroll sideways (board
  // «Tel-Dettagli»): 36px (control-md, pending Cowork), a 14px label on the caller's line, raised
  // and edged; the active one on accent-soft in accent text, no underline.
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
        [TabsSize.Large]:
          'h-tab-lg touch:h-control-md touch:rounded-full touch:border touch:border-line touch:bg-surface-raised touch:px-14 touch:text-cta-inline touch:font-medium touch:text-ink touch:data-[state=active]:border-accent-edge touch:data-[state=active]:bg-accent-soft touch:data-[state=active]:text-accent-text touch:data-[state=active]:shadow-none',
      },
    },
    defaultVariants: { size: TabsSize.Default },
  },
)
