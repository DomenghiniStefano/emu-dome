import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const CardVariant = {
  // An opaque card on a page.
  Surface: 'surface',
  // A card of the game details' tabs: Surface, and on the phone the panel's corners and the
  // phone's room inside (`card-pad-touch`), board «Tel-Dettagli».
  Detail: 'detail',
  // A card inside a card, or one of a choice of cards (a setting's choice cards); `selected` marks
  // the chosen one.
  Raised: 'raised',
  // Raised, and the chosen one's text turns accent too (Settings scope bar: «Tutti i giochi»,
  // «Giochi <console>», «Solo questo gioco»).
  RaisedAccent: 'raisedAccent',
  // One of a choice of pictures (the cover picker): opaque surface, `selected` on accent-soft with
  // an accent-text edge; `data-empty` (nothing to show yet, «add one») draws a dashed strong edge;
  // disabled (L12 «Da uno screenshot» with none yet, the reason in its text) fades to `dim`.
  PictureChoice: 'pictureChoice',
  // One of a few cards to choose from on a page (the controllers in Controls, the A/B layout of
  // the first run): Surface's fill, hovered, and the chosen one on accent-soft with an accent-text
  // edge.
  Choice: 'choice',
  // One row of a list the user picks from, rendered `as="button"` (Saves: the games): cover,
  // text and badge in a row, no fill until hovered, `selected` on accent-soft.
  ListItem: 'listItem',
  // A card that holds rows separated by lines (Controls: shortcut lists, the remap table; the
  // rows of ui/link-row): no padding, the rows draw their own.
  List: 'list',
  // The quick menu's cards as buttons (Trello #62). Their heights are off the design system's
  // scales: pending tokens, or literals of this one place.
  // A small choice among a row of them, on surface (a quick-save slot, G07); `selected` marks the
  // chosen one. On the phone a 2px edge when chosen.
  Tile: 'tile',
  // A tool to run or switch on and off (G07 Strumenti), its icon over its name; `selected` = on.
  // On the phone a row on surface: icon beside the name, 44 tall.
  Tool: 'tool',
  // A row that leads to another page: icon, name, chevron (G07, G08). Not ui/link-row: its own edge
  // and fill, control-lg tall.
  NavRow: 'navRow',
  // A line of a list, no edge of its own (G10): the list draws the box.
  ListRow: 'listRow',
} as const
export type CardVariant = (typeof CardVariant)[keyof typeof CardVariant]

// The room inside a card, when it is not its variant's own (Surface: `card-pad`, 10). Set with
// Surface or List; the other variants keep their own.
export const CardPad = {
  // 8 (`card-pad-md`): a stat, a line with its action, one side of a save conflict.
  Snug: 'snug',
  // 6 (`card-pad-sm`): a card packed with a picture and actions (a slot tile, «Continua a
  // giocare»), a banner with a button at its end.
  Tight: 'tight',
} as const
export type CardPad = (typeof CardPad)[keyof typeof CardPad]

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
        'rounded-card border border-line bg-surface p-card-pad-sm disabled:cursor-not-allowed disabled:opacity-dim data-empty:not-data-selected:border-dashed data-empty:not-data-selected:border-line-strong data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.Choice]:
        'rounded-card border border-line bg-surface px-10 py-8 enabled:hover:not-data-selected:bg-surface-hover data-selected:border-accent-text data-selected:bg-accent-soft',
      [CardVariant.ListItem]:
        'flex-row items-center gap-8 rounded-card border-0 bg-transparent p-6 hover:not-data-selected:bg-surface-hover data-selected:bg-accent-soft',
      [CardVariant.List]:
        'm-0 list-none overflow-hidden rounded-card border border-line bg-surface p-0',
      [CardVariant.Tile]:
        'rounded-card border border-line bg-surface p-4 data-selected:border-accent-text data-selected:bg-accent-soft touch:data-selected:border-strong',
      [CardVariant.Tool]:
        'h-tool items-center justify-center rounded-card border border-line bg-surface-raised px-8 text-center text-caption-inline font-semibold whitespace-nowrap enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-dim data-selected:border-accent-text data-selected:bg-accent-soft data-selected:text-accent-text touch:h-touch touch:flex-row touch:justify-start touch:bg-surface touch:px-10 touch:text-left touch:text-cta-inline touch:enabled:hover:bg-surface-hover touch:data-selected:bg-accent-soft',
      [CardVariant.NavRow]:
        'h-control-lg flex-row items-center rounded-card border border-line bg-surface px-10 text-ink enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-locked',
      [CardVariant.ListRow]:
        'min-h-control-lg flex-row items-center border-0 bg-transparent px-10 py-4 text-ink enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-locked',
    },
    pad: {
      [CardPad.Snug]: 'p-card-pad-md',
      [CardPad.Tight]: 'p-card-pad-sm',
    },
  },
  defaultVariants: { variant: CardVariant.Surface },
})
export type CardVariants = VariantProps<typeof cardVariants>
