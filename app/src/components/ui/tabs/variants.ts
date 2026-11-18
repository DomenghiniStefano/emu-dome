import { cva } from 'class-variance-authority'

export const TabsSize = {
  // A section's tabs (ControlsView: 40px).
  Default: 'default',
  // The game details' tabs (GameDetails: 44px).
  Large: 'large',
  // Pills that scroll sideways (the game details on the phone, board «Tel-Dettagli»): 36px, a
  // 14px label, raised and edged; the active one on accent-soft in accent text, no underline.
  // 36 and 14 are off the scale (spec Decision 5, Cowork). The label's line is the caller's.
  Pill: 'pill',
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
        [TabsSize.Pill]:
          'h-[36px] rounded-full border border-line bg-surface-raised px-14 text-[14px] font-medium text-ink data-[state=active]:border-accent-edge data-[state=active]:bg-accent-soft data-[state=active]:text-accent-text data-[state=active]:shadow-none',
      },
    },
    defaultVariants: { size: TabsSize.Default },
  },
)
