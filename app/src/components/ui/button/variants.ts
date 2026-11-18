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
  // An action inside a message (a toast's «Annulla», G17): raised fill without an edge, its label
  // in accent text.
  SoftAccent: 'softAccent',
  // A control of the on-screen gamepad (TouchControls): glass with a strong edge, darker while
  // pressed, no fade (a press must show at once).
  Touch: 'touch',
  // A quiet action inside a list row in body weight (first run S02 «Rimuovi»): Ghost at 400.
  GhostBody: 'ghostBody',
  // An action at the end of a list row (first run S06): Soft, half faded when it cannot be used
  // (0.5 and no pointer change, a value waiting for Cowork: spec Decision 5).
  SoftRow: 'softRow',
} as const
export type ButtonVariant = (typeof ButtonVariant)[keyof typeof ButtonVariant]

export const ButtonSize = {
  // 28px: actions inside a row (`.ghost`).
  Small: 'small',
  // 32px: the standard control (`.btn`).
  Default: 'default',
  // 40px: the main action (`.btn-primary`), with its 14px label.
  Large: 'large',
  // 36px: a dialog's actions (FlowDialog), the main one with a 13px label. Not one of the design
  // system's three heights: the fourth step (`control-md`) waits for Cowork (spec Decision 5), so
  // the value is written here, once.
  Medium: 'medium',
  // A toast's action (G17): 32px, 44 on a touch screen (the phone's touch minimum), radius 8.
  // 8 and 44 wait for Cowork (spec Decision 5), so they are written here, once.
  Toast: 'toast',
  // The on-screen gamepad (TouchControls), sized for a thumb: the shoulders L / R, the quick
  // menu, the round face buttons, Select / Start. Their sizes are the touch board's own, outside
  // the control heights (88 and 64 wait for Cowork).
  TouchShoulder: 'touchShoulder',
  TouchMenu: 'touchMenu',
  TouchFace: 'touchFace',
  TouchSystem: 'touchSystem',
  // 40px with the standard label (first run «Aggiungi una cartella», «Indietro», «Salta»).
  Tall: 'tall',
  // The phone's first run (Tel-PrimoAvvio) sets its text one step up, in lead type (15/22),
  // buttons included: the same heights in that type, plus its own footer (48, and 52 for the
  // main action). 48 and 52 wait for Cowork.
  SmallLead: 'smallLead',
  DefaultLead: 'defaultLead',
  TallLead: 'tallLead',
  FooterLead: 'footerLead',
  MainLead: 'mainLead',
  // Squares for an icon alone, at the two control heights.
  Icon: 'icon',
  IconSmall: 'iconSmall',
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
        [ButtonVariant.Touch]:
          'border-line-strong bg-glass font-semibold text-ink transition-none [-webkit-tap-highlight-color:transparent] touch-none active:bg-surface-hover',
        [ButtonVariant.GhostBody]:
          'border-0 bg-transparent font-regular text-ink-muted hover:bg-surface-hover disabled:hover:bg-transparent',
        [ButtonVariant.SoftRow]:
          'border-line bg-surface-raised text-ink hover:bg-surface-hover disabled:cursor-default disabled:opacity-50',
        [ButtonVariant.SoftAccent]:
          'border-0 bg-surface-raised font-semibold text-accent-text hover:bg-surface-hover',
      },
      size: {
        [ButtonSize.Small]: 'h-control-sm gap-4 px-8',
        [ButtonSize.Default]: 'h-control px-12',
        [ButtonSize.Large]: 'h-control-lg px-18 text-cta',
        [ButtonSize.Medium]: 'h-[36px] px-12',
        [ButtonSize.Tall]: 'h-control-lg px-16',
        [ButtonSize.SmallLead]: 'h-control-sm gap-4 px-8 text-lead',
        [ButtonSize.DefaultLead]: 'h-control px-12 text-lead font-medium',
        [ButtonSize.TallLead]: 'h-control-lg px-16 text-lead font-medium',
        [ButtonSize.FooterLead]: 'h-[48px] px-16 text-lead font-medium',
        [ButtonSize.MainLead]: 'h-[52px] px-18 text-cta',
        [ButtonSize.TouchShoulder]: 'h-control-lg w-[88px] rounded-10',
        [ButtonSize.TouchMenu]: 'h-control-lg w-48 rounded-10',
        [ButtonSize.TouchFace]: 'size-[64px] rounded-full text-dialog',
        [ButtonSize.TouchSystem]: 'h-control rounded-16 px-14 text-overline',
        [ButtonSize.Toast]:
          'h-control rounded-[8px] px-10 pointer-coarse:h-[44px] pointer-coarse:px-16',
        [ButtonSize.Icon]: 'size-control p-0',
        [ButtonSize.IconSmall]: 'size-control-sm p-0',
      },
    },
    // The main action at 36px keeps more room on its sides (FlowDialog: 16).
    compoundVariants: [{ variant: ButtonVariant.Primary, size: ButtonSize.Medium, class: 'px-16' }],
    defaultVariants: {
      variant: ButtonVariant.Soft,
      size: ButtonSize.Default,
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
