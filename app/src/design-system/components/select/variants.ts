import { cva } from 'class-variance-authority'

export const SelectTriggerVariant = {
  // The field showing the chosen option (SettingSelect `.field`): 200px at least, no wider than
  // its container, a control high, an edge on surface-raised, in the text colour around it; a
  // long value is cut with an ellipsis on its one line (SelectTrigger.vue).
  Field: 'field',
  // The choice inside a sentence or a title (Settings scope bar «Giochi <console> ▾»): no edge and
  // no fill until hovered, in the surrounding text's colour and semibold, 26px high (taller when
  // a narrow title wraps it, never over the line under it) and pulled 3px up and 10px left so its
  // text lines up with the text around it (off the scale: design).
  InTitle: 'inTitle',
} as const
export type SelectTriggerVariant = (typeof SelectTriggerVariant)[keyof typeof SelectTriggerVariant]

// Disabled (a locked setting's list, #132): faded like Input and DateTimeField.
export const selectTriggerVariants = cva(
  'flex cursor-pointer items-center gap-6 rounded-control border px-field-pad text-left font-sans text-label enabled:hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-disabled',
  {
    variants: {
      variant: {
        [SelectTriggerVariant.Field]:
          'h-control max-w-full min-w-field border-line bg-surface-raised text-inherit',
        [SelectTriggerVariant.InTitle]:
          '-mt-3 -ml-10 min-h-in-title min-w-0 border-transparent bg-transparent font-semibold text-inherit',
      },
    },
    defaultVariants: { variant: SelectTriggerVariant.Field },
  },
)
