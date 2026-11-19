import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const SearchFieldSize = {
  // 32px: a list's search (Saves V01 `.search`).
  Default: 'default',
} as const
export type SearchFieldSize = (typeof SearchFieldSize)[keyof typeof SearchFieldSize]

// The field is the <label>: the icon, the (screen reader) label and the bare <input> sit in it,
// so a click anywhere on the field focuses the input. The input keeps the browser's own small
// padding, as the screens' search fields always had.
export const searchFieldVariants = cva(
  'flex items-center gap-8 border border-line-strong bg-surface-raised text-ink-muted',
  {
    variants: {
      size: {
        [SearchFieldSize.Default]: 'h-control rounded-control px-field-pad',
      },
    },
    defaultVariants: { size: SearchFieldSize.Default },
  },
)
export type SearchFieldVariants = VariantProps<typeof searchFieldVariants>
