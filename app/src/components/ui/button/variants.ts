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
  // An icon alone on a page's bar, in ink with no fill until hovered (the game details' ‹ and «…»
  // on the phone, GameDetails `.icon-btn`).
  Plain: 'plain',
  // Words that act, muted until hovered: the way back above a page («‹ Libreria», GameDetails
  // `.back-link`).
  Text: 'text',
  // Words that act in accent text, underlined on hover; a disabled one is faint (DetailsSettings
  // `.link`, «Apri tutte le impostazioni di questo gioco»).
  Link: 'link',
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
  // 40px with the standard label: an action beside the main one (GameDetails `.btn.tall`).
  Tall: 'tall',
  // The phone's touch minimum (README: 44px), radius 10 (board «Tel-Dettagli» `.phone .btn`).
  // Its label takes the size and line of the text around it (the phone's type is larger), in the
  // medium weight, wrapping onto a second line when it is long.
  Touch: 'touch',
  // A 44px square for an icon alone on the phone (GameDetails `.icon-btn`).
  IconTouch: 'iconTouch',
  // The phone's full-width main action (GameDetails `.play.wide`): 52px, a 16px label on a 22px
  // line, radius 10. Off the scale (spec Decision 5, Cowork).
  Hero: 'hero',
  // No box at all, for Text and Link: the words alone, wrapping like text.
  Bare: 'bare',
} as const
export type ButtonSize = (typeof ButtonSize)[keyof typeof ButtonSize]

// Variants only colour and sizes only size: cva writes the size's classes after the variant's,
// so a height inside a variant would lose. Primary's hover is flat accent-hover: the dome's
// gradient is a background image, which `bg-none` drops (`.btn-primary:hover`). Disabled is the screens' `.btn:disabled`: 40% and the
// not-allowed cursor. The focus ring is the global one (styles.css, `data-nav`), not drawn here.
// Only the background fades, as `.btn` does (`transition: background 150ms`): `transition-colors`
// would also fade the ring's outline colour, which the pixel compare of the game details catches
// (Trello #62).
export const buttonVariants = cva(
  'inline-flex shrink-0 cursor-pointer items-center justify-center gap-6 whitespace-nowrap rounded-6 border border-transparent font-sans text-label transition-[background] select-none disabled:cursor-not-allowed disabled:opacity-disabled [&_svg]:shrink-0',
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
        [ButtonVariant.Plain]: 'border-0 bg-transparent text-ink hover:bg-surface-hover',
        [ButtonVariant.Text]: 'border-0 bg-transparent text-ink-muted hover:text-ink',
        [ButtonVariant.Link]:
          'border-0 bg-transparent text-accent-text enabled:hover:underline disabled:text-ink-faint disabled:opacity-100',
      },
      size: {
        [ButtonSize.Small]: 'h-control-sm gap-4 px-8',
        [ButtonSize.Default]: 'h-control px-12',
        [ButtonSize.Large]: 'h-control-lg px-18 text-cta',
        [ButtonSize.Icon]: 'size-control p-0',
        [ButtonSize.IconSmall]: 'size-control-sm p-0',
        [ButtonSize.Tall]: 'h-control-lg px-12',
        [ButtonSize.Touch]:
          'h-touch rounded-10 px-14 text-[length:inherit] leading-[inherit] font-medium whitespace-normal',
        [ButtonSize.IconTouch]: 'size-touch rounded-10 p-0',
        [ButtonSize.Hero]: 'h-[52px] gap-8 rounded-10 px-18 text-[16px]/[22px]',
        [ButtonSize.Bare]: 'h-auto gap-4 rounded-none p-0 whitespace-normal',
      },
    },
    defaultVariants: {
      variant: ButtonVariant.Soft,
      size: ButtonSize.Default,
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
