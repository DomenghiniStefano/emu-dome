import { cva } from 'class-variance-authority'

export const SelectTriggerVariant = {
  // The field showing the chosen option (SettingSelect `.field`): 200px at least, 32px high, an
  // edge on surface-raised.
  Field: 'field',
  // The choice inside a sentence or a title (Settings scope bar «Giochi <console> ▾»): no edge and
  // no fill until hovered, in the surrounding text's colour and semibold, 26px high and pulled
  // 3px up and 10px left so its text lines up with the text around it (off the scale: Cowork).
  InTitle: 'inTitle',
} as const
export type SelectTriggerVariant = (typeof SelectTriggerVariant)[keyof typeof SelectTriggerVariant]

export const selectTriggerVariants = cva(
  'flex cursor-pointer items-center gap-8 rounded-6 border px-10 text-left font-sans text-label enabled:hover:bg-surface-hover disabled:cursor-not-allowed',
  {
    variants: {
      variant: {
        [SelectTriggerVariant.Field]:
          'h-control min-w-field border-line bg-surface-raised text-ink',
        [SelectTriggerVariant.InTitle]:
          '-mt-[3px] -ml-10 h-[26px] min-w-0 border-transparent bg-transparent font-semibold text-inherit',
      },
    },
    defaultVariants: { variant: SelectTriggerVariant.Field },
  },
)
