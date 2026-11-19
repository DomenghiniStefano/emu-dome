import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

// How wide a section page's blocks grow: each direct child is held to it (the page itself fills
// its place, so its scrollbar stays at the window's edge).
export const SectionWidth = {
  // No limit (Saves, a controller's remap pages, E05).
  Full: 'full',
  // A settings page (`page`, 920).
  Page: 'page',
  // A game's details (`page-md`, 1040).
  Medium: 'medium',
  // Controls' overview (`page-wide`, 1180).
  Wide: 'wide',
} as const
export type SectionWidth = (typeof SectionWidth)[keyof typeof SectionWidth]

export const sectionPageVariants = cva(
  'flex min-w-0 flex-col gap-section-gap px-section-x pt-section-top pb-section-bottom touch:gap-section-touch touch:p-section-touch',
  {
    variants: {
      width: {
        [SectionWidth.Full]: '',
        [SectionWidth.Page]: '*:w-full *:max-w-page',
        [SectionWidth.Medium]: '*:w-full *:max-w-page-md',
        [SectionWidth.Wide]: '*:w-full *:max-w-page-wide',
      },
      // A page that scrolls itself (it fills the rest of its row); without, it grows with its
      // content inside a scroller of its parent (a game's details, under the hero).
      scroll: {
        true: 'min-h-0 flex-1 overflow-y-auto',
        false: '',
      },
    },
    defaultVariants: { width: SectionWidth.Full, scroll: true },
  },
)
export type SectionPageVariants = VariantProps<typeof sectionPageVariants>
