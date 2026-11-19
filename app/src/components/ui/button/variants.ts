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
  // The way back to the page above, over a page's title (ControlsView «‹ Controlli»): Ghost whose
  // text turns ink when hovered.
  Back: 'back',
  // An action inside a message (a toast's «Annulla», G17): raised fill without an edge, its label
  // in accent text, semibold. Not SoftAccent, whose fill is the accent's soft one.
  RaisedAccent: 'raisedAccent',
  // A control of the on-screen gamepad (TouchControls): glass with a strong edge, darker while
  // pressed, no fade (a press must show at once).
  Touch: 'touch',
  // A quiet action inside a list row in body weight (first run S02 «Rimuovi»): Ghost at 400.
  GhostBody: 'ghostBody',
  // An action at the end of a list row (first run S06): Soft, half faded when it cannot be used
  // (0.5 and no pointer change, a value waiting for Cowork: spec Decision 5).
  SoftRow: 'softRow',
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
  // As tall as its line of text, an icon leading (ControlsView «‹ Controlli»).
  TextLine: 'textLine',
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
        [ButtonVariant.Back]:
          'border-0 bg-transparent text-ink-muted hover:bg-surface-hover hover:text-ink',
        [ButtonVariant.Touch]:
          'border-line-strong bg-glass font-semibold text-ink transition-none [-webkit-tap-highlight-color:transparent] touch-none active:bg-surface-hover',
        [ButtonVariant.GhostBody]:
          'border-0 bg-transparent font-regular text-ink-muted hover:bg-surface-hover disabled:hover:bg-transparent',
        [ButtonVariant.SoftRow]:
          'border-line bg-surface-raised text-ink hover:bg-surface-hover disabled:cursor-default disabled:opacity-50',
        [ButtonVariant.RaisedAccent]:
          'border-0 bg-surface-raised font-semibold text-accent-text hover:bg-surface-hover',
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
        [ButtonSize.TextLine]: 'gap-2 py-2 pr-8 pl-4',
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
    // The main action keeps its 14px label and wide padding at the smaller heights, as
    // `.btn-primary` does wherever a screen sets only its height. A size that sets the medium
    // weight would win over Strong's semibold: Strong keeps it.
    compoundVariants: [
      { variant: ButtonVariant.Primary, size: ButtonSize.Compact, class: 'px-18 text-cta' },
      { variant: ButtonVariant.Primary, size: ButtonSize.Medium, class: 'px-16' },
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
