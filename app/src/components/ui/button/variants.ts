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
  // Settings rows (SettingRow): a row shows that it cannot be changed by fading its text, so the
  // buttons inside it are not faded again when disabled (`.btn`, `.ghost` there have no disabled
  // look of their own). RowSoft is Soft, RowGhost a Ghost in regular weight («Ripristina»,
  // «Rimuovi»), RowGhostAccent the GhostAccent («Modifica l’elenco», «Fatto»).
  RowSoft: 'rowSoft',
  RowGhost: 'rowGhost',
  RowGhostAccent: 'rowGhostAccent',
  // An entry of a side list of pages (SettingsView categories): no fill until hovered, the page
  // shown (`aria-current="page"`) in accent text on accent-soft. With ButtonSize.NavItem.
  NavItem: 'navItem',
  // A row of a card list that opens something (Settings T01 «Usate più spesso», «Modificate di
  // recente»): full width, a line between rows, regular muted text. With ButtonSize.LinkRow.
  LinkRow: 'linkRow',
  // A colour to pick (`general.accent`): a round swatch, its fill from the caller's `style`, the
  // chosen one (`aria-checked`) ringed in ink. With ButtonSize.IconSmall; like the row buttons,
  // not faded when disabled.
  Swatch: 'swatch',
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
  // 32px, 8px side padding and none above or below (a squeezed column shrinks it to its text),
  // the label at the start: an entry of a side list (ButtonVariant.NavItem).
  NavItem: 'navItem',
  // At least 56px, 8 / 16 padding, 16 between its parts: a list row (ButtonVariant.LinkRow).
  LinkRow: 'linkRow',
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
        [ButtonVariant.RowSoft]:
          'border-line bg-surface-raised text-ink hover:bg-surface-hover disabled:opacity-100',
        [ButtonVariant.RowGhost]:
          'border-0 bg-transparent font-regular text-ink-muted hover:bg-surface-hover disabled:opacity-100 disabled:hover:bg-transparent',
        [ButtonVariant.RowGhostAccent]:
          'border-0 bg-transparent text-accent-text hover:bg-surface-hover disabled:opacity-100 disabled:hover:bg-transparent',
        [ButtonVariant.NavItem]:
          'justify-start border-0 bg-transparent text-ink hover:bg-surface-hover aria-[current=page]:bg-accent-soft aria-[current=page]:text-accent-text',
        [ButtonVariant.LinkRow]:
          'w-full justify-start whitespace-normal rounded-none border-0 border-t border-line bg-transparent text-left font-regular text-ink-muted first:border-t-0 hover:bg-surface-hover',
        [ButtonVariant.Swatch]:
          'rounded-full border-0 aria-checked:outline-2 aria-checked:outline-offset-3 aria-checked:outline-ink aria-checked:outline-solid disabled:opacity-100',
      },
      size: {
        [ButtonSize.Small]: 'h-control-sm gap-4 px-8',
        [ButtonSize.Default]: 'h-control px-12',
        [ButtonSize.Large]: 'h-control-lg px-18 text-cta',
        [ButtonSize.Icon]: 'size-control p-0',
        [ButtonSize.IconSmall]: 'size-control-sm p-0',
        [ButtonSize.NavItem]: 'h-control gap-8 px-8 py-0',
        [ButtonSize.LinkRow]: 'min-h-link-row gap-16 px-16 py-8',
      },
    },
    defaultVariants: {
      variant: ButtonVariant.Soft,
      size: ButtonSize.Default,
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
