import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const ButtonVariant = {
  // The one main action of a view (README «Colore»: Gioca / Riprendi): the accent's dome with
  // its glow. `.btn-primary` in styles.css.
  Primary: 'primary',
  // Every other action: raised surface, thin edge. `.btn` in styles.css.
  Soft: 'soft',
  // A quiet action inside a row (SettingRow `.ghost`): no fill until hovered.
  Ghost: 'ghost',
  // The same, in accent text: «Scegli», «Fatto» beside a list (SettingRow `.ghost.accent`).
  GhostAccent: 'ghostAccent',
  // An action that throws something away (QuickSaveDialog `.btn.danger`): Soft in danger ink.
  Danger: 'danger',
  // A second action in accent ink on the accent's soft fill, quieter than Primary (SlotTile
  // «Gioca da qui»).
  SoftAccent: 'softAccent',
} as const
export type ButtonVariant = (typeof ButtonVariant)[keyof typeof ButtonVariant]

export const ButtonSize = {
  // 28px: actions inside a row (`.ghost`).
  Small: 'small',
  // 32px: the standard control (`.btn`).
  Default: 'default',
  // 40px: the main action (`.btn-primary`), with its 14px label.
  Large: 'large',
  // Squares for an icon alone, at the two control heights.
  Icon: 'icon',
  IconSmall: 'iconSmall',
  // 36px: a dialog's actions (FlowDialog) and a page's pair of actions (Saves E03). Off the control
  // scale: Cowork decides whether it folds into 32 or 40 or becomes a step (spec Decision 5);
  // until then this is the one place it is written. Primary is 16px wide on each side here.
  Medium: 'medium',
  // 28px with a 12px label on the document's 18px line, wrapping when the room is short: the
  // actions inside a tile (SlotTile).
  Compact: 'compact',
} as const
export type ButtonSize = (typeof ButtonSize)[keyof typeof ButtonSize]

// Variants only colour and sizes only size: cva writes the size's classes after the variant's,
// so a height inside a variant would lose. Primary's hover is flat accent-hover: the dome's
// gradient is a background image, which `bg-none` drops (`.btn-primary:hover`). Disabled is the screens' `.btn:disabled`: 40% and the
// not-allowed cursor. The focus ring is the global one (styles.css, `data-nav`), not drawn here.
export const buttonVariants = cva(
  'inline-flex shrink-0 cursor-pointer items-center justify-center gap-6 whitespace-nowrap rounded-6 border border-transparent font-sans text-label transition-colors select-none disabled:cursor-not-allowed disabled:opacity-disabled [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        [ButtonVariant.Primary]:
          'border-0 bg-accent-dome font-semibold text-on-accent shadow-glow-sm hover:bg-none hover:bg-accent-hover',
        [ButtonVariant.Soft]: 'border-line bg-surface-raised text-ink hover:bg-surface-hover',
        [ButtonVariant.Ghost]:
          'border-0 bg-transparent text-ink-muted hover:bg-surface-hover disabled:hover:bg-transparent',
        [ButtonVariant.GhostAccent]:
          'border-0 bg-transparent text-accent-text hover:bg-surface-hover disabled:hover:bg-transparent',
        [ButtonVariant.Danger]: 'border-line bg-surface-raised text-danger hover:bg-surface-hover',
        [ButtonVariant.SoftAccent]:
          'border-0 bg-accent-soft text-accent-text hover:bg-surface-hover',
      },
      size: {
        [ButtonSize.Small]: 'h-control-sm gap-4 px-8',
        [ButtonSize.Default]: 'h-control px-12',
        [ButtonSize.Large]: 'h-control-lg px-18 text-cta',
        [ButtonSize.Icon]: 'size-control p-0',
        [ButtonSize.IconSmall]: 'size-control-sm p-0',
        [ButtonSize.Medium]: 'h-[36px] px-12',
        [ButtonSize.Compact]:
          'h-control-sm px-10 text-caption leading-[18px] font-medium whitespace-normal',
      },
    },
    compoundVariants: [{ variant: ButtonVariant.Primary, size: ButtonSize.Medium, class: 'px-16' }],
    defaultVariants: {
      variant: ButtonVariant.Soft,
      size: ButtonSize.Default,
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
