import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

// The sizes a game's cover (GameCover.vue) is shown at. Each sets the width (the height follows
// the 2:3 shape), the corners, the room around the title and the title's type, so a screen never
// restyles a cover. The widths are pending tokens (`cover-*`, spacing.css).
export const CoverSize = {
  // As wide as its cell (the library's grid, the cover picker's choices): the title in heading.
  Fill: 'fill',
  // One of the cover picker's choices: Fill with a control's corners.
  Choice: 'choice',
  // A list row's thumbnail (Saves, a game in the list): 28px, no room around it.
  Thumb: 'thumb',
  // A thumbnail beside a title (Saves, the game's panel; the game details' looks): 48px.
  Row: 'row',
  // A small cover in a row of a few (first run S06 «Pronto»): 96px, 80 on the phone, the title in
  // 11px.
  Small: 'small',
  // The game details' big cover: 168px, 96 on the phone.
  Hero: 'hero',
} as const
export type CoverSize = (typeof CoverSize)[keyof typeof CoverSize]

// The title's 19px line is a pending token (`leading-cover`).
export const coverVariants = cva(
  'relative flex aspect-2/3 items-end overflow-hidden text-left text-on-cover',
  {
    variants: {
      size: {
        [CoverSize.Fill]: 'rounded-card p-12 text-heading leading-cover',
        [CoverSize.Choice]: 'rounded-control p-12 text-heading leading-cover',
        [CoverSize.Thumb]: 'w-cover-thumb rounded-badge p-0',
        [CoverSize.Row]: 'w-cover-row rounded-control p-0',
        [CoverSize.Small]:
          'w-cover-small rounded-card p-8 text-fine font-semibold touch:w-cover-small-touch',
        [CoverSize.Hero]:
          'w-cover-hero rounded-card p-12 text-heading leading-cover touch:w-cover-hero-touch',
      },
    },
    defaultVariants: { size: CoverSize.Fill },
  },
)
export type CoverVariants = VariantProps<typeof coverVariants>
