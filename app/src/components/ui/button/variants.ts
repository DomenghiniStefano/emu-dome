import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const ButtonVariant = {
  // The one main action of a view (README «Colore»: Gioca / Riprendi): the accent's dome with
  // its glow.
  Primary: 'primary',
  // Every other action: raised surface, thin edge.
  Soft: 'soft',
  // Soft with quiet text: an action that shows a current value, «Ordina per: …» (Library).
  Quiet: 'quiet',
  // The action a pair leans to (quick menu «Salva per questo gioco»): Soft on a strong edge, in
  // semibold.
  Strong: 'strong',
  // One of a row of picks that can each be on (quick menu G09 «Pixel nitidi», «TV CRT»): Soft,
  // the one on (`aria-pressed`) edged and written in accent text.
  Pick: 'pick',
  // An action that throws something away (a save's «Elimina», «Chiudi il gioco»): Soft in
  // danger ink.
  Danger: 'danger',
  // Confirming a step that throws progress away (quick menu G11 «Riavvia»): danger ink and edge.
  DangerOutline: 'dangerOutline',
  // A second action in accent ink on the accent's soft fill, quieter than Primary (a save
  // slot's «Gioca da qui»).
  SoftAccent: 'softAccent',
  // An action inside a message (a toast's «Annulla», G17): raised fill without an edge, its label
  // in accent text, semibold. Not SoftAccent, whose fill is the accent's soft one.
  RaisedAccent: 'raisedAccent',
  // A quiet action: no fill until hovered, muted text (the quick menu's back, a step ‹ ›).
  Ghost: 'ghost',
  // Ghost in body weight: an action inside a list row («Rimuovi», «Ripristina»).
  GhostRegular: 'ghostRegular',
  // Ghost in accent text: «Scegli», «Fatto» beside a list (a setting's chips).
  GhostAccent: 'ghostAccent',
  // An icon alone on a page's bar, in ink with no fill until hovered (the game details' ‹ and
  // «…» on the phone).
  Plain: 'plain',
  // Words that act, muted until hovered: the way back above a page («‹ Libreria»).
  Text: 'text',
  // Words that act in accent text, underlined on hover; a disabled one is faint («Altri slot e
  // salvataggi», «Apri tutte le impostazioni di questo gioco»).
  Link: 'link',
  // An item of a menu or a panel's list (a cover's menu, the library's filters): regular text,
  // its icon muted; a chosen one (`data-selected`) in accent text, semibold.
  Menu: 'menu',
  // An entry of a side list of pages (the settings categories): no fill until hovered, the page
  // shown (`aria-current="page"`) in accent text on accent-soft. With ButtonSize.NavItem.
  NavItem: 'navItem',
  // The head of such a list, its overview (the settings' «Panoramica»): NavItem in semibold.
  NavHead: 'navHead',
  // A row of a card list that opens something (ui/link-row: Settings T01, Controls C01): full
  // width, a line between rows, regular muted text; hovered only as a button (a `div` row only
  // says where a thing is done). With ButtonSize.LinkRow.
  LinkRow: 'linkRow',
  // A section of the shell's rail: no fill until hovered; the current one (`aria-current="page"`)
  // on accent-soft with the rail marker and its icon glowing. Only the fill fades: the focus ring
  // appears at once.
  Rail: 'rail',
  // The light button on a cover, «Gioca» (board L01): ink fill, ground-coloured text.
  Inverse: 'inverse',
  // A button over a cover, «…»: a veil of surface-raised.
  Veil: 'veil',
  // No look of its own: a click area over something that draws itself (a cover).
  Bare: 'bare',
  // A colour to pick (`general.accent`): a round swatch, its fill from the caller's `style`, the
  // chosen one (`aria-checked`) ringed in ink. With ButtonSize.IconSmall and ButtonFade.None.
  Swatch: 'swatch',
  // A control of the on-screen gamepad (components/touch), the style «Fumé» (`touch.tskin`): no
  // box of its own, it colours the shape drawn inside it (touch/PadShape.vue, which inherits
  // `fill` and `stroke`): a fill near the darkest ground and an opaque mid-grey edge, the fill
  // lighter while pressed or held down (`aria-pressed`, a «hold» button), light letters, no fade
  // (a press must show at once). The see-through is one opacity on the whole control
  // (`opacity-pad`, TouchControl). With the Pad sizes.
  PadSmoke: 'padSmoke',
  // The style «Contorno»: no fill, an opaque light edge, a grey fill while pressed.
  PadOutline: 'padOutline',
  // A section of the phone's tab bar (#29, at the bottom in portrait): muted icon over its name,
  // no fill; the current one (`aria-current="page"`) in accent text. With ButtonSize.PhoneTab.
  PhoneTab: 'phoneTab',
  // A section's tab in TV mode (#30, at the top instead of the rail): muted words, the current
  // one (`aria-current="page"`) an ink pill, not the accent (accent only on the main action).
  // With ButtonSize.TvTab.
  TvTab: 'tvTab',
  // A row of Settings in TV mode, and an option of its full-screen choice sheet (#30): full
  // width, its words at the start and its value at the end, no fill until focused or hovered
  // (then raised). With ButtonSize.TvRow.
  TvRow: 'tvRow',
  // A field that opens something on a click (the top bar's «Cerca giochi, azioni e impostazioni»,
  // X01): surface with a thin edge, its words muted, from the start. With ButtonSize.Compact.
  Field: 'field',
  // An icon alone on the top bar (the notifications' bell, X03): muted, no fill until hovered; the
  // one whose panel is open (`aria-expanded`) in accent text on accent-soft. With ButtonSize.Icon.
  BarIcon: 'barIcon',
  // A result of the search (X01): a full-width row, regular ink, the chosen one (focused) on
  // accent-soft. With ButtonSize.Result.
  Result: 'result',
} as const
export type ButtonVariant = (typeof ButtonVariant)[keyof typeof ButtonVariant]

// Every size has one look per density: the desktop's, and after `touch:` the phone's
// (src/assets/main.css). A size that is the same on both has no `touch:` class. Values off the
// design system's scales are pending tokens (assets/theme/*.css); the few literals left are one
// size's own, written here once.
export const ButtonSize = {
  // 24px (control-sm): actions inside a row. On the phone, lead's size.
  Small: 'small',
  // 28px (control): the standard control. On the phone, lead's size.
  Default: 'default',
  // 32px (control-lg): the main action, with its 14px label; on the phone 52px, the label 16 / 22
  // (Tel-Dettagli «Riprendi», Tel-PrimoAvvio «Continua»).
  Large: 'large',
  // 32px with the standard label (first run «Aggiungi una cartella», the game details' actions).
  // On the phone, lead's size.
  Tall: 'tall',
  // A step's way out beside the main action (first run «Indietro», «Salta»): Tall, and on the
  // phone 48px in lead's size.
  Footer: 'footer',
  // 30px (control-md, pending Cowork): a dialog's and a page's pair of actions. Primary is 12px
  // wide on each side here.
  Medium: 'medium',
  // 24px with the standard padding (the library's «Continua a giocare»).
  Compact: 'compact',
  // Squares for an icon alone, at the two control heights, and at the touch minimum (44: the
  // phone sheet's back, the game details' bar on the phone).
  Icon: 'icon',
  IconSmall: 'iconSmall',
  IconTouch: 'iconTouch',
  // An icon alone on a list row or a tile (Trello #104: a version's «Gioca questa», a folder's
  // «Togli»): the 24px square, and on the phone the touch minimum. Its 28px sibling is ToolIcon.
  IconRow: 'iconRow',
  // The play button of a game in the phone's list and on a compact grid's cover (#104, the owner
  // 2026-11-22: «Riprendi» too tall for the row): a 32 square (control-lg) drawn, no taller than
  // the row's two lines of text, while the touch still takes the 44 minimum: on the phone an
  // invisible 44 square centred on it (`before:`, whatever the edge's width). Positioned
  // (`relative`) for that area; a caller's `absolute` replaces it.
  RowAction: 'rowAction',
  // A section of the rail: 64 x 48, icon over name.
  Rail: 'rail',
  // The actions over a cover (control-md), «Gioca» and the square «…».
  CoverAction: 'coverAction',
  CoverActionIcon: 'coverActionIcon',
  // A menu item (control) and a row of a panel's list (at least control-sm), content from the
  // start; on the phone both at least the touch minimum (44).
  MenuItem: 'menuItem',
  MenuRow: 'menuRow',
  // Sized by its container's layout (an absolute click area).
  Area: 'area',
  // control-sm with a 12px label on the 18px line, wrapping when the room is short: the actions inside
  // a save slot's tile.
  TileAction: 'tileAction',
  // As tall as its line of text, an icon leading (Controls «‹ Controlli», the phone's
  // «‹ Impostazioni»); on the phone at least the touch minimum tall (#29).
  TextLine: 'textLine',
  // The quick menu's (Trello #62): the phone's 48 height is off the design system's scales,
  // pending a token.
  // «‹ Menu rapido»: control, tight on the chevron's side.
  Back: 'back',
  // ‹ › beside a value (G09): control, 44 on the phone.
  Step: 'step',
  // An action: control-lg in semibold; on the phone 48, the label at 14px.
  Menu: 'menu',
  // A compact action: control-md; on the phone 48, the label at 14px in medium weight.
  MenuCompact: 'menuCompact',
  // One of a pair side by side (the changes «in prova»): MenuCompact, 44 on the phone.
  MenuPair: 'menuPair',
  // The main action (Riprendi il gioco): control-lg and 260 wide, and on the phone 48 at 16px, as wide
  // as its place.
  MenuMain: 'menuMain',
  // A link's text alone, in the size around it.
  Text: 'text',
  // A link beside a section's label («Altri slot e salvataggi», «Imposta le velocità»): Text,
  // and on the phone at least 44 tall, at 13px.
  LabelLink: 'labelLink',
  // An action inside a card (the game details): Default, and on the phone the touch minimum,
  // its label in the size and line of the text around it (the phone's type is
  // larger), medium weight, wrapping onto a second line when it is long.
  CardAction: 'cardAction',
  // No box at all, for Text and Link: the words alone, wrapping like text; on the phone at least
  // the touch minimum tall, in lead's size.
  Bare: 'bare',
  // control, 8px side padding and none above or below (a squeezed column shrinks it to its text),
  // the label at the start: an entry of a side list (ButtonVariant.NavItem). On the phone at
  // least the touch minimum tall (it no longer shrinks: the list scrolls), in lead's size (#29).
  NavItem: 'navItem',
  // At least link-row, 6 / 12 padding, 12 between its parts: a list row (ButtonVariant.LinkRow).
  LinkRow: 'linkRow',
  // A toast's action (G17): control; 44 on the phone (the touch minimum).
  Toast: 'toast',
  // The on-screen gamepad (components/touch), compact (owner, 2026-11-22): each size is the box
  // that takes the touch, the shape drawn inside it (touch/PadShape.vue) may be smaller. The
  // shoulders L / R and the quick menu are as tall as the strip between a DS's screens
  // (`pad-strip`, 36), the round face buttons 48, Select / Start 60 × 32 (pending tokens).
  PadShoulder: 'padShoulder',
  PadMenu: 'padMenu',
  PadFace: 'padFace',
  PadSystem: 'padSystem',
  // The C06 touch controls editor's toolbar (Trello #29): a tool with its words (32, the touch
  // minimum on the phone), and one with its icon alone (a square of the same height).
  Tool: 'tool',
  ToolIcon: 'toolIcon',
  // A section of the phone's tab bar (#29): an equal share of the bar, at least the touch
  // minimum tall and wide, icon over name.
  PhoneTab: 'phoneTab',
  // TV mode (#30): a section's tab (52, a pill, the TV label's type), and an action seen from the
  // sofa (64, the TV action's type; Soft at the TV label's), the focused game's «Riprendi».
  TvTab: 'tvTab',
  Tv: 'tv',
  // A Settings row in TV mode (ButtonVariant.TvRow): at least 80, the TV label's type.
  TvRow: 'tvRow',
  // A settings page in TV mode's list (ButtonVariant.NavItem): 52, the TV label's type.
  TvNavItem: 'tvNavItem',
  // A search result (ButtonVariant.Result): at least control-lg, 8 between its parts, content from the
  // start.
  Result: 'result',
  // A notification (ButtonVariant.Result in the bell's panel, X03): its dot on the first line,
  // 8 between its parts, 6 / 8 padding.
  Notice: 'notice',
} as const
export type ButtonSize = (typeof ButtonSize)[keyof typeof ButtonSize]

// How a disabled button shows it, `disabled` or `aria-disabled` (one that cannot act but still
// takes a press, to say why: the game details' «Riprendi» when the file is missing). Normal: 40% (`opacity-disabled`). Dim: half faded
// (`opacity-dim`, pending Cowork: the quick menu's actions, the first run's row actions). None: not
// faded, because what holds it already says it cannot be used (a settings row fades its text, a
// link turns faint) or the look itself must not change (a colour swatch).
export const ButtonFade = {
  Normal: 'normal',
  Dim: 'dim',
  None: 'none',
} as const
export type ButtonFade = (typeof ButtonFade)[keyof typeof ButtonFade]

// Variants only colour, sizes only size, the fade only the disabled look: cva writes them in that
// order, so a height inside a variant would lose. Primary's hover is flat accent-hover: the dome's
// gradient is a background image, which `bg-none` drops. The focus ring is the global one
// (styles.css, `data-nav`), not drawn here. Only the background and the transform move
// (`transition-[background,transform]`: the press's give, motion.css): `transition-colors` would
// also fade the ring's outline colour, which the pixel compare of the game details catches
// (Trello #62).
export const buttonVariants = cva(
  'inline-flex shrink-0 cursor-pointer items-center justify-center gap-6 whitespace-nowrap rounded-button border border-transparent font-sans text-label transition-[background,transform] select-none disabled:cursor-not-allowed aria-disabled:cursor-not-allowed [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        [ButtonVariant.Primary]:
          'border-0 bg-accent-dome font-semibold text-on-accent shadow-glow-sm hover:bg-none hover:bg-accent-hover',
        [ButtonVariant.Soft]: 'border-line bg-surface-raised text-ink hover:bg-surface-hover',
        [ButtonVariant.Quiet]:
          'border-line bg-surface-raised font-regular text-ink-muted hover:bg-surface-hover',
        [ButtonVariant.Strong]:
          'border-line-strong bg-surface-raised font-semibold text-ink hover:bg-surface-hover',
        [ButtonVariant.Pick]:
          'border-line bg-surface-raised text-ink hover:bg-surface-hover aria-pressed:border-accent-text aria-pressed:text-accent-text',
        [ButtonVariant.Danger]: 'border-line bg-surface-raised text-danger hover:bg-surface-hover',
        [ButtonVariant.DangerOutline]:
          'border-danger bg-surface-raised text-danger hover:bg-surface-hover',
        [ButtonVariant.SoftAccent]:
          'border-0 bg-accent-soft text-accent-text hover:bg-surface-hover',
        [ButtonVariant.RaisedAccent]:
          'border-0 bg-surface-raised font-semibold text-accent-text hover:bg-surface-hover',
        [ButtonVariant.Ghost]:
          'border-0 bg-transparent text-ink-muted hover:bg-surface-hover disabled:hover:bg-transparent',
        [ButtonVariant.GhostRegular]:
          'border-0 bg-transparent font-regular text-ink-muted hover:bg-surface-hover disabled:hover:bg-transparent',
        [ButtonVariant.GhostAccent]:
          'border-0 bg-transparent text-accent-text hover:bg-surface-hover disabled:hover:bg-transparent',
        [ButtonVariant.Plain]: 'border-0 bg-transparent text-ink hover:bg-surface-hover',
        [ButtonVariant.Text]: 'border-0 bg-transparent text-ink-muted hover:text-ink',
        [ButtonVariant.Link]:
          'border-0 bg-transparent font-medium text-accent-text enabled:hover:underline disabled:text-ink-faint',
        [ButtonVariant.Menu]:
          'border-0 bg-transparent font-regular text-ink hover:bg-surface-hover data-selected:font-semibold data-selected:text-accent-text [&_svg]:text-ink-muted',
        [ButtonVariant.NavItem]:
          'justify-start border-0 bg-transparent text-ink hover:bg-surface-hover aria-[current=page]:bg-accent-soft aria-[current=page]:text-accent-text',
        [ButtonVariant.NavHead]:
          'justify-start border-0 bg-transparent font-semibold text-ink hover:bg-surface-hover aria-[current=page]:bg-accent-soft aria-[current=page]:text-accent-text',
        [ButtonVariant.LinkRow]:
          'w-full justify-start whitespace-normal rounded-none border-0 border-t border-line bg-transparent text-left font-regular text-ink-muted first:border-t-0 enabled:hover:bg-surface-hover [&:not(button)]:cursor-default',
        [ButtonVariant.Rail]:
          'border-0 bg-transparent text-ink-muted transition-[background,transform] hover:not-aria-[current=page]:bg-surface-hover aria-[current=page]:bg-accent-soft aria-[current=page]:text-accent-text aria-[current=page]:shadow-rail aria-[current=page]:[&_svg]:glow-icon',
        [ButtonVariant.Inverse]: 'border-0 bg-ink font-semibold text-bg',
        [ButtonVariant.Veil]: 'border-0 bg-cover-action text-ink',
        [ButtonVariant.Bare]: 'border-0 bg-transparent',
        [ButtonVariant.Swatch]:
          'rounded-full border-0 aria-checked:outline-ring aria-checked:outline-offset-swatch aria-checked:outline-ink aria-checked:outline-solid',
        [ButtonVariant.PadSmoke]:
          'border-0 bg-transparent fill-pad-smoke stroke-pad-smoke-edge text-pad-ink transition-none [-webkit-tap-highlight-color:transparent] touch-none active:fill-pad-smoke-held aria-pressed:fill-pad-smoke-held',
        [ButtonVariant.PadOutline]:
          'border-0 bg-transparent fill-none stroke-pad-outline-edge text-pad-ink transition-none [-webkit-tap-highlight-color:transparent] touch-none active:fill-pad-outline-held aria-pressed:fill-pad-outline-held',
        [ButtonVariant.PhoneTab]:
          'border-0 bg-transparent text-ink-muted [-webkit-tap-highlight-color:transparent] aria-[current=page]:text-accent-text',
        [ButtonVariant.TvTab]:
          'border-0 bg-transparent text-ink-muted hover:text-ink aria-[current=page]:bg-ink aria-[current=page]:font-semibold aria-[current=page]:text-bg',
        [ButtonVariant.TvRow]:
          'w-full justify-between border-0 bg-transparent text-left font-regular whitespace-normal text-ink hover:bg-surface-hover focus:bg-surface-raised',
        [ButtonVariant.Field]:
          'justify-start border-line bg-surface font-regular text-ink-muted hover:bg-surface-hover',
        [ButtonVariant.BarIcon]:
          'relative border-0 bg-transparent text-ink-muted hover:bg-surface-hover aria-expanded:bg-accent-soft aria-expanded:text-accent-text',
        [ButtonVariant.Result]:
          'w-full justify-start border-0 bg-transparent text-left font-regular whitespace-normal text-ink hover:bg-surface-hover focus:bg-accent-soft disabled:hover:bg-transparent',
      },
      size: {
        [ButtonSize.Small]: 'h-control-sm gap-4 px-6 touch:text-label-lead',
        [ButtonSize.Default]: 'h-control px-control-pad touch:text-label-lead',
        [ButtonSize.Large]: 'h-control-lg px-14 text-cta touch:h-52 touch:gap-8 touch:text-hero',
        [ButtonSize.Tall]: 'h-control-lg px-12 touch:text-label-lead',
        [ButtonSize.Footer]: 'h-control-lg px-12 touch:h-48 touch:text-label-lead',
        [ButtonSize.Medium]: 'h-control-md px-control-pad py-0',
        [ButtonSize.Compact]: 'h-control-sm px-control-pad py-0',
        [ButtonSize.Icon]: 'size-control p-0',
        [ButtonSize.IconSmall]: 'size-control-sm p-0',
        [ButtonSize.IconTouch]: 'size-touch p-0',
        [ButtonSize.IconRow]: 'size-control-sm p-0 touch:size-touch',
        [ButtonSize.RowAction]:
          'relative size-control-lg p-0 touch:before:absolute touch:before:top-1/2 touch:before:left-1/2 touch:before:size-touch touch:before:-translate-1/2',
        [ButtonSize.Rail]: 'h-48 w-rail-item flex-col gap-3 p-0',
        [ButtonSize.CoverAction]: 'h-control-md px-6 py-px',
        [ButtonSize.CoverActionIcon]: 'size-control-md px-6 py-px',
        [ButtonSize.MenuItem]:
          'h-control justify-start gap-8 px-8 text-left touch:h-touch touch:text-label-lead',
        [ButtonSize.MenuRow]:
          'min-h-control-sm justify-start gap-8 px-6 text-left touch:min-h-touch',
        [ButtonSize.Area]: 'p-0',
        [ButtonSize.TileAction]:
          'h-control-sm px-8 text-caption-tall font-medium whitespace-normal',
        [ButtonSize.TextLine]: 'gap-2 py-2 pr-8 pl-4 touch:min-h-touch',
        [ButtonSize.Back]: 'h-control gap-4 pr-8 pl-4',
        [ButtonSize.Step]: 'size-control p-0 touch:size-touch',
        [ButtonSize.Menu]: 'h-control-lg gap-6 px-12 font-semibold touch:h-48 touch:text-cta',
        [ButtonSize.MenuCompact]:
          'h-control-md gap-6 px-12 touch:h-48 touch:text-cta touch:font-medium',
        [ButtonSize.MenuPair]:
          'h-control-md gap-6 px-12 touch:h-touch touch:text-cta touch:font-medium',
        [ButtonSize.MenuMain]:
          'h-control-lg w-menu-main gap-6 px-14 text-cta touch:h-48 touch:w-auto touch:text-hero',
        [ButtonSize.Text]: 'h-auto p-0 text-[length:inherit] whitespace-normal',
        [ButtonSize.LabelLink]:
          'h-auto p-0 text-[length:inherit] whitespace-normal touch:min-h-touch touch:text-label',
        [ButtonSize.CardAction]:
          'h-control px-control-pad touch:h-touch touch:px-12 touch:text-[length:inherit] touch:leading-[inherit] touch:font-medium touch:whitespace-normal',
        [ButtonSize.Bare]:
          'h-auto gap-4 rounded-none p-0 whitespace-normal touch:min-h-touch touch:text-label-lead',
        [ButtonSize.NavItem]:
          'h-control gap-6 px-8 py-0 touch:h-auto touch:min-h-touch touch:text-label-lead',
        [ButtonSize.LinkRow]: 'min-h-link-row gap-12 px-12 py-6',
        [ButtonSize.Toast]: 'h-control px-8 touch:h-touch touch:px-16',
        [ButtonSize.PadShoulder]: 'relative h-pad-strip w-pad-shoulder p-0 text-pad-shoulder',
        [ButtonSize.PadMenu]: 'relative size-pad-strip p-0',
        [ButtonSize.PadFace]: 'relative size-pad-face rounded-full p-0 text-pad-face',
        [ButtonSize.PadSystem]: 'relative h-pad-system-h w-pad-system p-0 text-pad-system',
        [ButtonSize.Tool]: 'h-control gap-6 px-8 touch:h-touch touch:px-12',
        [ButtonSize.ToolIcon]: 'size-control p-0 touch:size-touch',
        [ButtonSize.PhoneTab]: 'min-h-touch min-w-touch flex-1 basis-0 flex-col gap-2 px-2 py-4',
        [ButtonSize.TvTab]: 'h-tv-tab rounded-full px-24 text-tv-label',
        [ButtonSize.TvRow]:
          'min-h-tv-row gap-tv-row-x rounded-card px-tv-row-x py-tv-row-y text-tv-label',
        [ButtonSize.TvNavItem]: 'h-tv-tab gap-12 rounded-card px-tv-row-y text-tv-label',
        [ButtonSize.Tv]: 'h-tv-action gap-12 rounded-card px-32 text-tv-action',
        [ButtonSize.Result]: 'min-h-control-lg gap-8 px-8 py-4',
        [ButtonSize.Notice]: 'items-start gap-8 rounded-card px-8 py-6',
      },
      fade: {
        [ButtonFade.Normal]: 'disabled:opacity-disabled aria-disabled:opacity-disabled',
        [ButtonFade.Dim]: 'disabled:opacity-dim aria-disabled:opacity-dim',
        [ButtonFade.None]: '',
      },
    },
    // The main action keeps its 14px label and wide padding at the smaller heights. A size that
    // sets the medium weight on the phone would win over Strong's semibold: Strong keeps it.
    compoundVariants: [
      { variant: ButtonVariant.Primary, size: ButtonSize.Compact, class: 'px-12 text-cta' },
      { variant: ButtonVariant.Primary, size: ButtonSize.Medium, class: 'px-12' },
      { variant: ButtonVariant.Soft, size: ButtonSize.Tv, class: 'px-28 text-tv-label' },
      {
        variant: ButtonVariant.Strong,
        size: [ButtonSize.MenuCompact, ButtonSize.MenuPair],
        class: 'touch:font-semibold',
      },
    ],
    defaultVariants: {
      variant: ButtonVariant.Soft,
      size: ButtonSize.Default,
      fade: ButtonFade.Normal,
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
