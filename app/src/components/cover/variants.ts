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
  // As wide as its cell in the phone's compact grid (#29): no title on it (the tile writes it
  // under the cover), the ROM icon larger and centred.
  Compact: 'compact',
  // A list row's thumbnail (Saves, a game in the list): 28px, no room around it.
  Thumb: 'thumb',
  // A row of the library's list (#29): 44px, no title, the ROM icon larger and centred.
  ListRow: 'listRow',
  // A thumbnail beside a title (Saves, the game's panel; the game details' looks): 48px.
  Row: 'row',
  // A small cover in a row of a few (first run S06 «Pronto»): 96px, 80 on the phone, the title in
  // 11px.
  Small: 'small',
  // The game details' big cover: 168px, 96 on the phone.
  Hero: 'hero',
  // TV mode (#30): a shelf's cover, as wide as its click area (168, 200 focused), the title in
  // the TV cover's type; the focused game's big cover on top, as tall as its band.
  Shelf: 'shelf',
  TvHero: 'tvHero',
} as const
export type CoverSize = (typeof CoverSize)[keyof typeof CoverSize]

// The title's 19px line is a pending token (`leading-cover`).
export const coverVariants = cva(
  'relative flex aspect-2/3 items-end overflow-hidden text-left text-on-cover',
  {
    variants: {
      size: {
        [CoverSize.Fill]: 'rounded-card p-10 text-heading leading-cover',
        [CoverSize.Choice]: 'rounded-control p-10 text-heading leading-cover',
        [CoverSize.Compact]: 'rounded-card p-0',
        [CoverSize.Thumb]: 'w-cover-thumb rounded-badge p-0',
        [CoverSize.ListRow]: 'w-cover-list flex-none rounded-badge p-0',
        [CoverSize.Row]: 'w-cover-row rounded-control p-0',
        [CoverSize.Small]:
          'w-cover-small rounded-card p-8 text-fine font-semibold touch:w-cover-small-touch',
        [CoverSize.Hero]:
          'w-cover-hero rounded-card p-10 text-heading leading-cover touch:w-cover-hero-touch',
        [CoverSize.Shelf]: 'w-full rounded-card p-14 text-tv-cover',
        [CoverSize.TvHero]: 'h-full rounded-panel p-16 text-tv-heading',
      },
    },
    defaultVariants: { size: CoverSize.Fill },
  },
)
export type CoverVariants = VariantProps<typeof coverVariants>

// A game in several regional versions (#102, owner R1): the edges of two more covers behind its
// tile, as a deck, the near one wider and lower. Opaque surfaces, so the theme decides them.
export const CoverStackLayer = {
  Near: 'near',
  Far: 'far',
} as const
export type CoverStackLayer = (typeof CoverStackLayer)[keyof typeof CoverStackLayer]

export const coverStackVariants = cva(
  'pointer-events-none absolute h-full rounded-card border border-line',
  {
    variants: {
      layer: {
        [CoverStackLayer.Near]: 'inset-x-5 -top-5 bg-surface-hover',
        [CoverStackLayer.Far]: 'inset-x-10 -top-10 bg-surface-raised',
      },
    },
  },
)

// The «N versioni» tag (#102, owner R1): square like the console badge, its look and type, in
// the opposite corner.
const TV_VERSIONS = 'top-10 right-10 px-8 text-tv-badge'
export const coverVersionsVariants = cva('absolute rounded-badge bg-cover-badge py-px', {
  variants: {
    size: {
      [CoverSize.Fill]: 'top-8 right-8 px-6 text-console-badge',
      [CoverSize.Choice]: 'top-8 right-8 px-6 text-console-badge',
      [CoverSize.Compact]: 'top-6 right-6 px-4 text-console-badge',
      [CoverSize.Thumb]: 'top-8 right-8 px-6 text-console-badge',
      [CoverSize.ListRow]: 'top-8 right-8 px-6 text-console-badge',
      [CoverSize.Row]: 'top-8 right-8 px-6 text-console-badge',
      [CoverSize.Small]: 'top-8 right-8 px-6 text-console-badge',
      [CoverSize.Hero]: 'top-8 right-8 px-6 text-console-badge',
      [CoverSize.Shelf]: TV_VERSIONS,
      [CoverSize.TvHero]: TV_VERSIONS,
    },
  },
  defaultVariants: { size: CoverSize.Fill },
})

// The console badge in a corner, per cover size: the small type everywhere, the TV type on a
// shelf's cover and the focused game's big one (TV mode, #30: no text under 18px).
const TV_BADGE = 'top-10 left-10 px-8 text-tv-badge'
export const coverBadgeVariants = cva('absolute rounded-badge bg-cover-badge py-px', {
  variants: {
    size: {
      [CoverSize.Fill]: 'top-8 left-8 px-6 text-console-badge',
      [CoverSize.Choice]: 'top-8 left-8 px-6 text-console-badge',
      [CoverSize.Compact]: 'top-6 left-6 px-4 text-console-badge',
      [CoverSize.Thumb]: 'top-8 left-8 px-6 text-console-badge',
      [CoverSize.ListRow]: 'top-8 left-8 px-6 text-console-badge',
      [CoverSize.Row]: 'top-8 left-8 px-6 text-console-badge',
      [CoverSize.Small]: 'top-8 left-8 px-6 text-console-badge',
      [CoverSize.Hero]: 'top-8 left-8 px-6 text-console-badge',
      [CoverSize.Shelf]: TV_BADGE,
      [CoverSize.TvHero]: TV_BADGE,
    },
  },
  defaultVariants: { size: CoverSize.Fill },
})

// The title over a cover without a picture, held to a few lines per size so a long one never
// climbs under the console badge (the whole title stays under the cover and in the details).
// TV mode keeps its own look (not clamped).
export const coverTitleVariants = cva('relative wrap-anywhere', {
  variants: {
    size: {
      [CoverSize.Fill]: 'line-clamp-4',
      [CoverSize.Choice]: 'line-clamp-4',
      [CoverSize.Compact]: 'line-clamp-3',
      [CoverSize.Thumb]: 'line-clamp-2',
      [CoverSize.ListRow]: 'line-clamp-2',
      [CoverSize.Row]: 'line-clamp-2',
      [CoverSize.Small]: 'line-clamp-3',
      [CoverSize.Hero]: 'line-clamp-4',
      [CoverSize.Shelf]: '',
      [CoverSize.TvHero]: '',
    },
  },
  defaultVariants: { size: CoverSize.Fill },
})

// The ROM icon (small pixel art, 32×32 on NDS), scaled up crisp: in the upper middle, above the
// title; on the sizes that never write a title (the phone's compact grid and list, #29) larger
// and centred, so the cover is not a big empty colour around a small icon.
const ICON_OVER_TITLE = 'top-[38%] w-2/5'
const ICON_ALONE = 'top-1/2 w-3/5'
export const coverIconVariants = cva('pixelated absolute left-1/2 -translate-1/2 rounded-control', {
  variants: {
    size: {
      [CoverSize.Fill]: ICON_OVER_TITLE,
      [CoverSize.Choice]: ICON_OVER_TITLE,
      [CoverSize.Compact]: ICON_ALONE,
      [CoverSize.Thumb]: ICON_OVER_TITLE,
      [CoverSize.ListRow]: ICON_ALONE,
      [CoverSize.Row]: ICON_OVER_TITLE,
      [CoverSize.Small]: ICON_OVER_TITLE,
      [CoverSize.Hero]: ICON_OVER_TITLE,
      [CoverSize.Shelf]: ICON_OVER_TITLE,
      [CoverSize.TvHero]: ICON_OVER_TITLE,
    },
  },
  defaultVariants: { size: CoverSize.Fill },
})
