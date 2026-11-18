import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const CardVariant = {
  // An opaque card on a page (Details: `.card` in DetailsFiles, DetailsControls).
  Surface: 'surface',
  // A card inside a card, or one of a choice of cards (SettingRow `.card`); `selected` marks the
  // chosen one.
  Raised: 'raised',
  // A glass panel over the ambient haze (README «Vetro»: thick glass for menus and panels).
  Glass: 'glass',
  // The quick menu's cards as buttons (Trello #62). Their 44 / 48 / 52 / 68 heights, 12 radius
  // and the disabled 0.5 are off the design system's scales: kept until Cowork decides (spec
  // Decision 5), each in this one place.
  // A small choice among a row of them, on surface (a quick-save slot, G07); `selected` marks the
  // chosen one. On the phone a 12 radius and a 2px edge when chosen.
  Tile: 'tile',
  TileTouch: 'tileTouch',
  // A tool to run or switch on and off (G07 Strumenti), its icon over its name; `selected` = on.
  // On the phone a row: icon beside the name.
  Tool: 'tool',
  ToolTouch: 'toolTouch',
  // A row that leads to another page: icon, name, chevron (G07, G08).
  Row: 'row',
  // A line of a list, no edge of its own (G10): the list draws the box.
  ListRow: 'listRow',
} as const
export type CardVariant = (typeof CardVariant)[keyof typeof CardVariant]

// The gap between the card's parts is the caller's: the screens use 6, 10 and 14 today.
export const cardVariants = cva('flex flex-col text-left', {
  variants: {
    variant: {
      [CardVariant.Surface]: 'rounded-10 border border-line bg-surface p-16',
      [CardVariant.Raised]:
        'rounded-10 border border-line bg-surface-raised p-10 data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.Glass]: 'glass-thick p-24',
      [CardVariant.Tile]:
        'rounded-10 border border-line bg-surface p-6 data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.TileTouch]:
        'rounded-[12px] border border-line bg-surface p-6 data-selected:border-2 data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.Tool]:
        'h-[68px] items-center justify-center rounded-10 border border-line bg-surface-raised px-8 text-center text-(length:--text-caption) font-semibold whitespace-nowrap enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-[0.5] data-selected:border-accent-text data-selected:bg-accent-soft data-selected:text-accent-text',
      [CardVariant.ToolTouch]:
        'h-[44px] flex-row items-center rounded-[12px] border border-line bg-surface px-10 text-(length:--text-cta) font-semibold whitespace-nowrap enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-[0.5] data-selected:border-accent-text data-selected:bg-accent-soft data-selected:text-accent-text',
      [CardVariant.Row]:
        'h-48 flex-row items-center rounded-10 border border-line bg-surface px-12 text-ink enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-locked',
      [CardVariant.ListRow]:
        'min-h-[52px] flex-row items-center border-0 bg-transparent px-12 py-8 text-ink enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-locked',
    },
  },
  defaultVariants: { variant: CardVariant.Surface },
})
export type CardVariants = VariantProps<typeof cardVariants>
