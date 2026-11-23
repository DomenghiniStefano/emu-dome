import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

// A heading's look, by what it heads. The tag is the caller's (`as`: h1, h2, h3, div, span), so
// the document outline stays right whatever the look.
export const HeadingVariant = {
  // The small uppercase label over a card, a list or a group of controls («SALVATAGGI RAPIDI»,
  // «PROVA I PULSANTI»), in muted ink; one step larger on the phone (`overline-lg`).
  Overline: 'overline',
  // A heading inside a page, over a group of its parts (the settings' search results).
  Heading: 'heading',
  // A page's or a block's title (Library, Saves, Settings, Controls, an empty state).
  Title: 'title',
  // A quick menu page's title (`page-title`, pending design).
  PageTitle: 'pageTitle',
  // A scene's title (a first run step, a game starting or closing): display type, smaller on
  // the phone (`display-md`).
  Display: 'display',
  // A game's name beside its cover (the game details): display type, and on the phone the
  // smaller `display-sm`, so a long name fits beside the cover.
  Hero: 'hero',
} as const
export type HeadingVariant = (typeof HeadingVariant)[keyof typeof HeadingVariant]

export const headingVariants = cva('m-0', {
  variants: {
    variant: {
      [HeadingVariant.Overline]: 'text-overline text-ink-muted uppercase touch:text-overline-lg',
      [HeadingVariant.Heading]: 'text-heading',
      [HeadingVariant.Title]: 'text-title',
      [HeadingVariant.PageTitle]: 'text-page-title',
      [HeadingVariant.Display]: 'text-display touch:text-display-md',
      [HeadingVariant.Hero]: 'text-display wrap-anywhere touch:text-display-sm',
    },
  },
  defaultVariants: { variant: HeadingVariant.Title },
})
export type HeadingVariants = VariantProps<typeof headingVariants>
