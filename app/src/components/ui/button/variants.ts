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
  // Soft with quiet text: an action that shows a current value, «Ordina per: …» (LibraryView).
  Quiet: 'quiet',
  // A section of the shell's rail (AppShell): no fill until hovered; the current one
  // (`aria-current="page"`) on accent-soft with the rail marker and its icon glowing. Only the
  // fill fades (`.rail-item`): the focus ring appears at once.
  Rail: 'rail',
  // The light button on a cover, «Gioca» (board L01): ink fill, ground-coloured text.
  Inverse: 'inverse',
  // A button over a cover, «…»: a veil of surface-raised.
  Veil: 'veil',
  // An item of a menu or a panel's list (CoverMenu, LibraryFilters): regular text, its icon
  // muted; a chosen one (`data-selected`) in accent text, semibold.
  Menu: 'menu',
  // No look of its own: a click area over something that draws itself (a cover).
  Bare: 'bare',
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
  // 28px with the standard padding (LibraryView «Continua a giocare»: `.btn` at 28).
  Compact: 'compact',
  // 36px: LibraryEmpty, CoverPicker, a dialog's actions (FlowDialog) and a page's pair of actions
  // (Saves E03). Off the control scale: Cowork decides whether it folds into 32 or 40 or becomes a
  // step (spec Decision 5); until then this is the one place it is written. Primary is 16px wide
  // on each side here.
  Medium: 'medium',
  // A section of the rail: 64 x 52, icon over name (radius 8 and gap 3 await Cowork).
  Rail: 'rail',
  // The actions over a cover (36, radius 8: Cowork), «Gioca» and the square «…».
  CoverAction: 'coverAction',
  CoverActionIcon: 'coverActionIcon',
  // A menu item (32) and a row of a panel's list (at least 28), content from the start.
  MenuItem: 'menuItem',
  MenuRow: 'menuRow',
  // Sized by its container's layout (an absolute click area).
  Area: 'area',
  // 28px with a 12px label on the document's 18px line, wrapping when the room is short: the
  // actions inside a tile (SlotTile).
  TileAction: 'tileAction',
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
        [ButtonVariant.Quiet]:
          'border-line bg-surface-raised font-regular text-ink-muted hover:bg-surface-hover',
        [ButtonVariant.Rail]:
          'border-0 bg-transparent text-ink-muted transition-[background] hover:not-aria-[current=page]:bg-surface-hover aria-[current=page]:bg-accent-soft aria-[current=page]:text-accent-text aria-[current=page]:shadow-rail aria-[current=page]:[&_svg]:glow-icon',
        [ButtonVariant.Inverse]: 'border-0 bg-ink font-semibold text-bg',
        [ButtonVariant.Veil]: 'border-0 bg-cover-action text-ink',
        [ButtonVariant.Menu]:
          'border-0 bg-transparent font-regular text-ink hover:bg-surface-hover data-selected:font-semibold data-selected:text-accent-text [&_svg]:text-ink-muted',
        [ButtonVariant.Bare]: 'border-0 bg-transparent',
        [ButtonVariant.SoftAccent]:
          'border-0 bg-accent-soft text-accent-text hover:bg-surface-hover',
      },
      size: {
        [ButtonSize.Small]: 'h-control-sm gap-4 px-8',
        [ButtonSize.Default]: 'h-control px-12',
        [ButtonSize.Large]: 'h-control-lg px-18 text-cta',
        [ButtonSize.Icon]: 'size-control p-0',
        [ButtonSize.IconSmall]: 'size-control-sm p-0',
        [ButtonSize.Compact]: 'h-control-sm px-12 py-0',
        [ButtonSize.Medium]: 'h-[36px] px-12 py-0',
        [ButtonSize.Rail]: 'h-[52px] w-[64px] flex-col gap-[3px] rounded-[8px] p-0',
        [ButtonSize.CoverAction]: 'h-[36px] rounded-[8px] px-6 py-px',
        [ButtonSize.CoverActionIcon]: 'size-[36px] rounded-[8px] px-6 py-px',
        [ButtonSize.MenuItem]: 'h-control justify-start gap-8 px-10 text-left',
        [ButtonSize.MenuRow]: 'min-h-control-sm justify-start gap-8 px-6 text-left',
        [ButtonSize.Area]: 'p-0',
        [ButtonSize.TileAction]:
          'h-control-sm px-10 text-caption leading-[18px] font-medium whitespace-normal',
      },
    },
    // The main action keeps its 14px label and wide padding at the smaller heights, as
    // `.btn-primary` does wherever a screen sets only its height.
    compoundVariants: [
      { variant: ButtonVariant.Primary, size: ButtonSize.Compact, class: 'px-18 text-cta' },
      { variant: ButtonVariant.Primary, size: ButtonSize.Medium, class: 'px-16' },
    ],
    defaultVariants: {
      variant: ButtonVariant.Soft,
      size: ButtonSize.Default,
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
