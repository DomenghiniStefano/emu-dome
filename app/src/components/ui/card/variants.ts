import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const CardVariant = {
  // An opaque card on a page.
  Surface: 'surface',
  // A card of the game details' tabs: Surface, and on the phone rounder (16) and a little tighter
  // (14 inside), board «Tel-Dettagli».
  Detail: 'detail',
  // A card inside a card, or one of a choice of cards (a setting's choice cards); `selected` marks
  // the chosen one.
  Raised: 'raised',
  // Raised, and the chosen one's text turns accent too (Settings scope bar: «Tutti i giochi»,
  // «Giochi <console>», «Solo questo gioco»).
  RaisedAccent: 'raisedAccent',
  // One of a choice of pictures (the cover picker): opaque surface, `selected` on accent-soft with
  // an accent-text edge; `data-empty` (nothing to show yet, «add one») draws a dashed strong edge.
  PictureChoice: 'pictureChoice',
  // One of a few cards to choose from on a page (the controllers in Controls, the A/B layout of
  // the first run): Surface's fill, hovered, and the chosen one on accent-soft with an accent-text
  // edge.
  Choice: 'choice',
  // One row of a list the user picks from, rendered `as="button"` (Saves: the games): cover,
  // text and badge in a row, no fill until hovered, `selected` on accent-soft.
  ListItem: 'listItem',
  // A card that holds rows separated by lines (Controls: shortcut lists, the remap table): no
  // padding, the rows draw their own.
  List: 'list',
  // A row of a List card that opens what it names (Controls «Rimappa i pulsanti»): at least 56
  // tall, 16 between its parts, no frame of its own, hovered when it is a button; a disabled one is
  // half faded (`opacity-dim`).
  Row: 'row',
  // The quick menu's cards as buttons (Trello #62). Their 44 / 48 / 52 / 68 heights and 12 radius
  // are off the design system's scales: pending tokens, or literals of this one place.
  // A small choice among a row of them, on surface (a quick-save slot, G07); `selected` marks the
  // chosen one. On the phone a 12 radius and a 2px edge when chosen.
  Tile: 'tile',
  // A tool to run or switch on and off (G07 Strumenti), its icon over its name; `selected` = on.
  // On the phone a row on surface: icon beside the name, 44 tall.
  Tool: 'tool',
  // A row that leads to another page: icon, name, chevron (G07, G08). Not Row: its own edge
  // and fill, 48 tall.
  NavRow: 'navRow',
  // A line of a list, no edge of its own (G10): the list draws the box.
  ListRow: 'listRow',
} as const
export type CardVariant = (typeof CardVariant)[keyof typeof CardVariant]

// The gap between the card's parts is the caller's: the screens use 6, 10 and 14 today. A `touch:`
// class that changes a fill repeats the states after it (`touch:data-selected:…`): the touch
// variant comes later in the stylesheet than a state alone, so it would cover the state's fill.
export const cardVariants = cva('flex flex-col text-left', {
  variants: {
    variant: {
      [CardVariant.Surface]: 'rounded-card border border-line bg-surface p-card-pad',
      [CardVariant.Detail]:
        'rounded-card border border-line bg-surface p-card-pad touch:rounded-panel touch:p-card-pad-touch',
      [CardVariant.Raised]:
        'rounded-card border border-line bg-surface-raised p-card-pad-sm data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.RaisedAccent]:
        'rounded-card border border-line bg-surface-raised p-card-pad-sm data-selected:border-accent-text data-selected:bg-accent-soft data-selected:text-accent-text',
      [CardVariant.PictureChoice]:
        'rounded-card border border-line bg-surface p-card-pad-sm data-empty:not-data-selected:border-dashed data-empty:not-data-selected:border-line-strong data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.Choice]:
        'rounded-card border border-line bg-surface px-14 py-12 enabled:hover:not-data-selected:bg-surface-hover data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.ListItem]:
        'flex-row items-center gap-10 rounded-8 border-0 bg-transparent p-8 hover:not-data-selected:bg-surface-hover data-selected:bg-accent-soft',
      [CardVariant.List]: 'overflow-hidden rounded-card border border-line bg-surface',
      [CardVariant.Row]:
        'min-h-link-row flex-row items-center gap-16 border-0 bg-transparent px-16 py-8 enabled:hover:bg-surface-hover disabled:cursor-default disabled:opacity-dim',
      [CardVariant.Tile]:
        'rounded-card border border-line bg-surface p-6 data-selected:border-accent-text data-selected:bg-accent-soft touch:rounded-12 touch:data-selected:border-strong',
      [CardVariant.Tool]:
        'h-tool items-center justify-center rounded-card border border-line bg-surface-raised px-8 text-center text-caption-inline font-semibold whitespace-nowrap enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-dim data-selected:border-accent-text data-selected:bg-accent-soft data-selected:text-accent-text touch:h-touch touch:flex-row touch:justify-start touch:rounded-12 touch:bg-surface touch:px-10 touch:text-left touch:text-cta-inline touch:enabled:hover:bg-surface-hover touch:data-selected:bg-accent-soft',
      [CardVariant.NavRow]:
        'h-48 flex-row items-center rounded-card border border-line bg-surface px-12 text-ink enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-locked',
      [CardVariant.ListRow]:
        'min-h-52 flex-row items-center border-0 bg-transparent px-12 py-8 text-ink enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-locked',
    },
  },
  defaultVariants: { variant: CardVariant.Surface },
})
export type CardVariants = VariantProps<typeof cardVariants>
