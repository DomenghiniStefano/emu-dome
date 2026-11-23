import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const DialogTone = {
  // A question or a notice: the title in ink.
  Default: 'default',
  // Something failed (G02 cannot start, E11 the emulator stopped).
  Danger: 'danger',
  // Something needs a look but nothing is lost (E04 the save cannot be read, E12 slow).
  Warning: 'warning',
} as const
export type DialogTone = (typeof DialogTone)[keyof typeof DialogTone]

export const DialogSize = {
  // A question with its answers (FlowDialog): 520px at most, centred.
  Default: 'default',
  // A choice laid out wide (L12 «Scegli la copertina»): the panel's width, the screen behind
  // blurred; the panel scrolls when the window is short. The 8px blur and the 640px width are
  // not design system values yet (design, spec Decision 5).
  Wide: 'wide',
  // The search everywhere (X01, #32): the panel's width, near the top of the window
  // (`palette-top`, the whole width on the phone), no padding of its own (its rows are edge to
  // edge); its list scrolls when the window is short.
  Palette: 'palette',
  // A choice from the sofa (Settings in TV mode, #30, TvChoiceSheet): the whole screen, the
  // glass over the blurred screen behind, no edge, corners or padding of its own (the sheet
  // lays out its column).
  TvSheet: 'tvSheet',
} as const
export type DialogSize = (typeof DialogSize)[keyof typeof DialogSize]

// One column as wide as the window less its gutter (`minmax(0,1fr)`): a panel never grows past
// it, whatever its content (a long path without a break point, #29 on a 412 px phone).
export const dialogOverlayVariants = cva('fixed inset-0 z-dialog grid bg-scrim p-12', {
  variants: {
    size: {
      [DialogSize.Default]: 'grid-cols-[minmax(0,1fr)] place-items-center',
      [DialogSize.Wide]: 'grid-cols-[minmax(0,1fr)] place-items-center backdrop-blur-dim',
      [DialogSize.Palette]:
        'grid-cols-[minmax(0,1fr)] items-start justify-items-center pt-palette-top touch:pt-12',
      [DialogSize.TvSheet]: 'grid-cols-[minmax(0,1fr)] p-0 backdrop-blur-dim',
    },
  },
  defaultVariants: { size: DialogSize.Default },
})

export const dialogContentVariants = cva('glass-thick flex flex-col', {
  variants: {
    size: {
      [DialogSize.Default]: 'w-full max-w-dialog gap-8 p-dialog-pad',
      [DialogSize.Wide]: 'max-h-full w-panel max-w-full gap-8 overflow-auto p-dialog-pad',
      [DialogSize.Palette]: 'max-h-full w-panel max-w-full gap-0 overflow-hidden p-0',
      [DialogSize.TvSheet]:
        'size-full min-h-0 gap-0 overflow-hidden rounded-none border-0 p-0 shadow-none',
    },
  },
  defaultVariants: { size: DialogSize.Default },
})

export const DialogTitleSize = {
  // A dialog's question (FlowDialog).
  Dialog: 'dialog',
  // The title over a wide choice (L12), in the heading step.
  Heading: 'heading',
  // The title of a full-screen choice from the sofa (DialogSize.TvSheet), and its description
  // (DialogDescriptionSize.Tv): the TV display and lead types.
  Tv: 'tv',
} as const
export type DialogTitleSize = (typeof DialogTitleSize)[keyof typeof DialogTitleSize]

export const DialogDescriptionSize = {
  // Under a dialog's title: the body's type.
  Default: 'default',
  // Under a TV sheet's title (DialogTitleSize.Tv): the TV lead's type.
  Tv: 'tv',
} as const
export type DialogDescriptionSize =
  (typeof DialogDescriptionSize)[keyof typeof DialogDescriptionSize]

// A word longer than the line (a path, a file name) breaks anywhere rather than overflowing.
export const dialogDescriptionVariants = cva('m-0 text-ink-muted wrap-anywhere', {
  variants: {
    size: {
      [DialogDescriptionSize.Default]: 'text-body',
      [DialogDescriptionSize.Tv]: 'text-tv-lead',
    },
  },
  defaultVariants: { size: DialogDescriptionSize.Default },
})

export const dialogTitleVariants = cva('m-0 wrap-anywhere', {
  variants: {
    tone: {
      [DialogTone.Default]: 'text-ink',
      [DialogTone.Danger]: 'text-danger',
      [DialogTone.Warning]: 'text-warning',
    },
    size: {
      [DialogTitleSize.Dialog]: 'text-dialog',
      [DialogTitleSize.Heading]: 'text-heading',
      [DialogTitleSize.Tv]: 'text-tv-display',
    },
  },
  defaultVariants: { tone: DialogTone.Default, size: DialogTitleSize.Dialog },
})
export type DialogTitleVariants = VariantProps<typeof dialogTitleVariants>
