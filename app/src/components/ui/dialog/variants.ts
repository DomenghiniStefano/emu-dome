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
  // not design system values yet (Cowork, spec Decision 5).
  Wide: 'wide',
  // The search everywhere (X01, #32): the panel's width, near the top of the window
  // (`palette-top`, the whole width on the phone), no padding of its own (its rows are edge to
  // edge); its list scrolls when the window is short.
  Palette: 'palette',
} as const
export type DialogSize = (typeof DialogSize)[keyof typeof DialogSize]

export const dialogOverlayVariants = cva('fixed inset-0 z-dialog grid bg-scrim p-16', {
  variants: {
    size: {
      [DialogSize.Default]: 'place-items-center',
      [DialogSize.Wide]: 'place-items-center backdrop-blur-dim',
      [DialogSize.Palette]:
        'grid-cols-[minmax(0,1fr)] items-start justify-items-center pt-palette-top touch:pt-16',
    },
  },
  defaultVariants: { size: DialogSize.Default },
})

export const dialogContentVariants = cva('glass-thick flex flex-col', {
  variants: {
    size: {
      [DialogSize.Default]: 'w-full max-w-dialog gap-12 p-dialog-pad',
      [DialogSize.Wide]: 'max-h-full w-panel max-w-full gap-14 overflow-auto p-24',
      [DialogSize.Palette]: 'max-h-full w-panel max-w-full gap-0 overflow-hidden p-0',
    },
  },
  defaultVariants: { size: DialogSize.Default },
})

export const DialogTitleSize = {
  // A dialog's question (FlowDialog).
  Dialog: 'dialog',
  // The title over a wide choice (L12), in the heading step.
  Heading: 'heading',
} as const
export type DialogTitleSize = (typeof DialogTitleSize)[keyof typeof DialogTitleSize]

export const dialogTitleVariants = cva('m-0', {
  variants: {
    tone: {
      [DialogTone.Default]: 'text-ink',
      [DialogTone.Danger]: 'text-danger',
      [DialogTone.Warning]: 'text-warning',
    },
    size: {
      [DialogTitleSize.Dialog]: 'text-dialog',
      [DialogTitleSize.Heading]: 'text-heading',
    },
  },
  defaultVariants: { tone: DialogTone.Default, size: DialogTitleSize.Dialog },
})
export type DialogTitleVariants = VariantProps<typeof dialogTitleVariants>
