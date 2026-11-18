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
  // The action a pair leans to (quick menu «Salva per questo gioco»): Soft on a strong edge, in
  // semibold.
  Strong: 'strong',
  // Confirming a step that throws progress away (quick menu G11 «Riavvia»): danger ink and edge.
  DangerOutline: 'dangerOutline',
  // A link beside a heading or inside a sentence (quick menu «Altri slot e salvataggi»): accent
  // text, no fill, no edge.
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
  // The quick menu's sizes (Trello #62). Their 36 / 44 / 48 heights, 12 radius and the disabled
  // 0.5 / 0.35 are off the design system's scales: kept as they are until Cowork decides (spec
  // Decision 5), each in this one place.
  // «‹ Menu rapido»: 32px, tight on the chevron's side.
  Back: 'back',
  // ‹ › beside a value (G09): 32, and 44 on the phone.
  Step: 'step',
  StepTouch: 'stepTouch',
  // An icon alone at the phone's touch minimum, 44 (the sheet's back).
  IconTouch: 'iconTouch',
  // An action: 44 (the touch minimum, README) in semibold, 36 compact; on the phone 48, with the
  // label at 14px; side by side on the phone, 44.
  Menu: 'menu',
  MenuCompact: 'menuCompact',
  MenuTouch: 'menuTouch',
  MenuCompactTouch: 'menuCompactTouch',
  MenuPairTouch: 'menuPairTouch',
  // The main action (Riprendi il gioco): 44, and 48 on the phone at 16px.
  MenuMain: 'menuMain',
  MenuMainTouch: 'menuMainTouch',
  // A link's text alone, in the size around it; on the phone at least 44 tall, at 13px.
  Text: 'text',
  TextTouch: 'textTouch',
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
        [ButtonVariant.Strong]:
          'border-line-strong bg-surface-raised font-semibold text-ink hover:bg-surface-hover',
        [ButtonVariant.DangerOutline]:
          'border-danger bg-surface-raised text-danger hover:bg-surface-hover',
        [ButtonVariant.Link]: 'border-0 bg-transparent font-medium text-accent-text',
      },
      size: {
        [ButtonSize.Small]: 'h-control-sm gap-4 px-8',
        [ButtonSize.Default]: 'h-control px-12',
        [ButtonSize.Large]: 'h-control-lg px-18 text-cta',
        [ButtonSize.Icon]: 'size-control p-0',
        [ButtonSize.IconSmall]: 'size-control-sm p-0',
        [ButtonSize.Back]: 'h-control gap-4 pr-10 pl-4',
        [ButtonSize.Step]: 'size-control p-0 disabled:opacity-[0.35]',
        [ButtonSize.StepTouch]: 'size-[44px] p-0 disabled:opacity-[0.35]',
        [ButtonSize.IconTouch]: 'size-[44px] rounded-10 p-0',
        [ButtonSize.Menu]: 'h-[44px] gap-8 rounded-10 px-14 font-semibold disabled:opacity-[0.5]',
        [ButtonSize.MenuCompact]: 'h-[36px] gap-8 rounded-10 px-14 disabled:opacity-[0.5]',
        [ButtonSize.MenuTouch]: 'h-48 gap-8 rounded-[12px] px-14 text-cta disabled:opacity-[0.5]',
        [ButtonSize.MenuCompactTouch]:
          'h-48 gap-8 rounded-[12px] px-14 text-cta font-medium disabled:opacity-[0.5]',
        [ButtonSize.MenuPairTouch]:
          'h-[44px] gap-8 rounded-[12px] px-14 text-cta font-medium disabled:opacity-[0.5]',
        [ButtonSize.MenuMain]: 'h-[44px] gap-8 rounded-10 px-18 text-cta',
        [ButtonSize.MenuMainTouch]: 'h-48 gap-8 rounded-[12px] px-18 text-[16px]',
        [ButtonSize.Text]: 'h-auto p-0 text-[length:inherit] whitespace-normal',
        [ButtonSize.TextTouch]:
          'h-auto min-h-[44px] p-0 text-(length:--text-label) whitespace-normal',
      },
    },
    // A size that sets the medium weight would win over Strong's semibold: Strong keeps it.
    compoundVariants: [
      {
        variant: ButtonVariant.Strong,
        size: [ButtonSize.MenuCompactTouch, ButtonSize.MenuPairTouch],
        class: 'font-semibold',
      },
    ],
    defaultVariants: {
      variant: ButtonVariant.Soft,
      size: ButtonSize.Default,
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
