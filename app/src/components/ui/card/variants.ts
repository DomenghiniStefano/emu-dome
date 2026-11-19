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
  // One of a choice of pictures (CoverPicker): opaque surface, `selected` on accent-soft with an
  // accent-text edge; `data-empty` (nothing to show yet, «add one») draws a dashed strong edge.
  PictureChoice: 'pictureChoice',
  // One row of a list the user picks from, rendered `as="button"` (Saves: the games): cover,
  // text and badge in a row, no fill until hovered, `selected` on accent-soft. Its 8px radius is
  // off the scale, kept until Cowork decides (spec Decision 5: 8 → 6).
  ListItem: 'listItem',
  // A card that holds rows separated by lines (Controls: shortcut lists, the remap table): no
  // padding, the rows draw their own.
  List: 'list',
  // One of a few cards to choose from on a page (ControlsOverview: the devices): Surface's fill,
  // hovered, and the chosen one on accent-soft with an accent-text edge.
  Choice: 'choice',
  // A row of a List card that opens what it names (ControlsOverview «Rimappa i pulsanti»): no
  // frame of its own, hovered when it is a button; a disabled one is faded (0.5, spec Decision 5:
  // `locked` 0.55 awaits Cowork).
  Row: 'row',
  // One of a choice of cards on a page (first run S04, A/B layout): Surface with 14 inside;
  // `selected` marks the chosen one as Raised does. Not Choice: no hover, 14 on every side.
  LayoutChoice: 'layoutChoice',
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
  // A row that leads to another page: icon, name, chevron (G07, G08). Not Row: its own edge
  // and fill, 48 tall.
  NavRow: 'navRow',
  // A line of a list, no edge of its own (G10): the list draws the box.
  ListRow: 'listRow',
  // Raised, and the chosen one's text turns accent too (Settings scope bar: «Tutti i giochi»,
  // «Giochi <console>», «Solo questo gioco»).
  RaisedAccent: 'raisedAccent',
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
      [CardVariant.PictureChoice]:
        'rounded-10 border border-line bg-surface p-10 data-empty:not-data-selected:border-dashed data-empty:not-data-selected:border-line-strong data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.ListItem]:
        'flex-row items-center gap-10 rounded-[8px] border-0 bg-transparent p-8 hover:not-data-selected:bg-surface-hover data-selected:bg-accent-soft',
      [CardVariant.List]: 'overflow-hidden rounded-10 border border-line bg-surface',
      [CardVariant.Choice]:
        'rounded-10 border border-line bg-surface px-14 py-12 enabled:hover:not-data-selected:bg-surface-hover data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.Row]:
        'flex-row items-center border-0 bg-transparent px-16 py-8 enabled:hover:bg-surface-hover disabled:cursor-default disabled:opacity-50',
      [CardVariant.LayoutChoice]:
        'rounded-10 border border-line bg-surface p-14 data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.Tile]:
        'rounded-10 border border-line bg-surface p-6 data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.TileTouch]:
        'rounded-[12px] border border-line bg-surface p-6 data-selected:border-2 data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.Tool]:
        'h-[68px] items-center justify-center rounded-10 border border-line bg-surface-raised px-8 text-center text-(length:--text-caption) font-semibold whitespace-nowrap enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-[0.5] data-selected:border-accent-text data-selected:bg-accent-soft data-selected:text-accent-text',
      [CardVariant.ToolTouch]:
        'h-[44px] flex-row items-center rounded-[12px] border border-line bg-surface px-10 text-(length:--text-cta) font-semibold whitespace-nowrap enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-[0.5] data-selected:border-accent-text data-selected:bg-accent-soft data-selected:text-accent-text',
      [CardVariant.NavRow]:
        'h-48 flex-row items-center rounded-10 border border-line bg-surface px-12 text-ink enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-locked',
      [CardVariant.ListRow]:
        'min-h-[52px] flex-row items-center border-0 bg-transparent px-12 py-8 text-ink enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-locked',
      [CardVariant.RaisedAccent]:
        'rounded-10 border border-line bg-surface-raised p-10 data-selected:border-accent-text data-selected:bg-accent-soft data-selected:text-accent-text',
    },
  },
  defaultVariants: { variant: CardVariant.Surface },
})
export type CardVariants = VariantProps<typeof cardVariants>
