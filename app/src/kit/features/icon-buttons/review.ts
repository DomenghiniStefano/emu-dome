// Kit review board of Trello #104 (development only): the app's text-only buttons, one group per
// area, each button as it is now and as proposed, both drawn with the real Button. The texts are
// the Italian ones of it.json written as plain text (the Kit is not translated, see KitPage.vue);
// the full list, file by file, is the card's inventory. Nothing here is read by the app.
import type { Component } from 'vue'
import {
  ArchiveRestore,
  ArrowLeftRight,
  ArrowRight,
  Blend,
  Check,
  CheckCheck,
  ChevronDown,
  ChevronLeft,
  ClipboardCopy,
  Clock,
  CloudUpload,
  Copy,
  File as FileIcon,
  FolderOpen,
  FolderPlus,
  FunnelX,
  Gamepad2,
  HardDrive,
  History as HistoryIcon,
  Hourglass,
  Image as ImageIcon,
  Import,
  Keyboard,
  Library,
  LogIn,
  LogOut,
  Pencil,
  Pin,
  Play,
  Plus,
  RefreshCw,
  Repeat2,
  RotateCcw,
  Save,
  Scaling,
  Share,
  SkipForward,
  Trash2,
  Undo2,
  X,
  Zap,
} from '@lucide/vue'
import { ButtonSize, ButtonVariant } from '@/components/ui/button'

export interface ButtonSpec {
  text: string
  variant?: ButtonVariant
  size?: ButtonSize
  icon?: Component
  filled?: boolean
  /** The icon alone, `text` its label (aria-label and tooltip). */
  iconOnly?: boolean
}

export interface ButtonPair {
  now: ButtonSpec
  next: ButtonSpec
}

export interface ReviewRow {
  /** What the buttons sit beside in a list row (a game, a folder), drawn as the row's words. */
  context?: string
  /** Drawn at the phone's touch density. */
  phone?: boolean
  buttons: ButtonPair[]
}

export interface ReviewGroup {
  title: string
  problem: string
  solution: string
  rows: ReviewRow[]
}

/** A button as it is, and the same with an icon before its words. */
function withIcon(now: ButtonSpec, icon: Component, filled = false): ButtonPair {
  return { now, next: { ...now, icon, filled } }
}

/** A button as it is, and the same as its icon alone at a square size (and maybe another look). */
function iconOnly(
  now: ButtonSpec,
  icon: Component,
  size: ButtonSize,
  opts: { filled?: boolean; variant?: ButtonVariant } = {},
): ButtonPair {
  return {
    now,
    next: {
      ...now,
      icon,
      size,
      iconOnly: true,
      filled: opts.filled ?? false,
      variant: opts.variant ?? now.variant,
    },
  }
}

/** A button that already has its icon, shown the same on both sides for context. */
function same(spec: ButtonSpec): ButtonPair {
  return { now: spec, next: spec }
}

const P = ButtonVariant.Primary
const M = ButtonSize.Medium

export const REVIEW_GROUPS: ReviewGroup[] = [
  {
    title: 'Libreria: righe del telefono, «Continua a giocare», libreria vuota, avvisi',
    problem:
      'Sul telefono ogni riga della lista ripete «Riprendi» in parole; la scheda «Continua a giocare», la libreria vuota e gli avvisi hanno solo testo. 16 pulsanti solo testo: 7 icona e testo, 1 solo icona, 8 restano (zone cliccabili sulle copertine, voci di ordinamento e filtri, i pulsanti TV che hanno già il glifo del controller).',
    solution:
      'la riga del telefono mostra il triangolo «Riprendi» da solo, come la griglia compatta (CompactTile) già fa; gli altri prendono l’icona davanti alle parole.',
    rows: [
      {
        context: 'Pokémon Smeraldo · GBA',
        phone: true,
        buttons: [
          iconOnly(
            { text: 'Riprendi', variant: P, size: ButtonSize.Tool },
            Play,
            ButtonSize.ToolIcon,
            {
              filled: true,
            },
          ),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Riprendi', variant: P, size: ButtonSize.Compact }, Play, true),
          withIcon({ text: 'Salvataggi', size: ButtonSize.Compact }, Save),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Scegli una cartella…', variant: P, size: M }, FolderOpen),
          withIcon({ text: 'Apri un file…', size: M }, FileIcon),
          withIcon({ text: 'Togli i filtri' }, FunnelX),
          withIcon({ text: 'Scegli un’altra cartella' }, FolderOpen),
        ],
      },
    ],
  },
  {
    title: 'Dettagli del gioco: azioni delle schede, versioni, impostazioni del gioco',
    problem:
      'Le azioni delle schede (DetailsAction) sono parole; ogni riga di «Versioni» ripete «Gioca questa» e «Mostra questa in libreria», ogni impostazione del gioco ripete «Ripristina». 5 pulsanti solo testo nel codice (DetailsAction, uno solo, fa 5 azioni): 1 icona e testo, 3 solo icona, 1 resta (il link alle impostazioni).',
    solution:
      'DetailsAction passa l’icona al pulsante (icona e testo); nelle righe resta la sola icona, il testo di oggi come etichetta e suggerimento.',
    rows: [
      {
        buttons: [
          withIcon(
            { text: 'Rimappa i pulsanti per questo gioco', size: ButtonSize.CardAction },
            Gamepad2,
          ),
          withIcon({ text: 'Mostra i backup', size: ButtonSize.CardAction }, HistoryIcon),
          withIcon({ text: 'Importa…', size: ButtonSize.CardAction }, Import),
          withIcon({ text: 'Esporta…', size: ButtonSize.CardAction }, Share),
          withIcon({ text: 'Cambia…', size: ButtonSize.CardAction }, ImageIcon),
        ],
      },
      {
        context: 'Pokémon Rubino (EUR)',
        buttons: [
          iconOnly({ text: 'Gioca questa', size: ButtonSize.Small }, Play, ButtonSize.IconRow, {
            filled: true,
          }),
          iconOnly(
            {
              text: 'Mostra questa in libreria',
              variant: ButtonVariant.GhostRegular,
              size: ButtonSize.Small,
            },
            Pin,
            ButtonSize.IconRow,
          ),
        ],
      },
      {
        context: 'Riavvolgimento · Questo gioco',
        buttons: [
          iconOnly(
            { text: 'Ripristina', variant: ButtonVariant.GhostRegular, size: ButtonSize.Small },
            Undo2,
            ButtonSize.IconRow,
          ),
        ],
      },
    ],
  },
  {
    title: 'Salvataggi: slot, backup, finestra dello slot, importazione',
    problem:
      'Ogni slot ripete «Gioca da qui» in parole accanto al suo «…» che è già un’icona; ogni backup ripete «Ripristina»; finestre e pagina di importazione sono solo testo. 21 pulsanti solo testo: 19 icona e testo, 2 solo icona.',
    solution:
      'nello slot e nella lista dei backup la sola icona; nelle finestre e nell’importazione icona e testo.',
    rows: [
      {
        context: 'Slot 2 · oggi 14:02',
        buttons: [
          iconOnly(
            {
              text: 'Gioca da qui',
              variant: ButtonVariant.SoftAccent,
              size: ButtonSize.TileAction,
            },
            Play,
            ButtonSize.IconRow,
            { filled: true },
          ),
        ],
      },
      {
        context: 'Ieri, 21:40 · prima di un caricamento',
        buttons: [iconOnly({ text: 'Ripristina' }, ArchiveRestore, ButtonSize.ToolIcon)],
      },
      {
        buttons: [
          withIcon({ text: 'Elimina', size: M }, Trash2),
          withIcon({ text: 'Esporta…', size: M }, Share),
          withIcon({ text: 'Chiudi', size: M }, X),
          withIcon({ text: 'Gioca da qui', variant: P, size: M }, Play, true),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Scegli un file…' }, FileIcon),
          withIcon({ text: 'Scegli il gioco…' }, Gamepad2),
          withIcon({ text: 'Annulla' }, X),
          withIcon({ text: 'Importa', variant: P, size: ButtonSize.Large }, Import),
        ],
      },
    ],
  },
  {
    title: 'Menu rapido: «Riprendi», modifiche in prova, conferme',
    problem:
      'Il «Riprendi» del menu rapido sul telefono è solo testo (sul desktop ha già il triangolo); le modifiche «in prova» e l’«Annulla» del riavvio sono parole accanto a pulsanti con icona. 9 pulsanti solo testo: 5 icona e testo, 4 restano (i link accanto alle etichette e le scelte di G09).',
    solution: '«Riprendi» con il triangolo anche sul telefono; icona e testo per gli altri.',
    rows: [
      {
        phone: true,
        buttons: [
          withIcon({ text: 'Riprendi', variant: P, size: ButtonSize.MenuMain }, Play, true),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Annulla le modifiche', size: ButtonSize.MenuPair }, Undo2),
          withIcon(
            {
              text: 'Salva per questo gioco',
              variant: ButtonVariant.Strong,
              size: ButtonSize.MenuPair,
            },
            Save,
          ),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Annulla', size: ButtonSize.Menu }, X),
          same({
            text: 'Riavvia',
            variant: ButtonVariant.DangerOutline,
            size: ButtonSize.Menu,
            icon: RefreshCw,
          }),
          withIcon({ text: 'Annulla', variant: ButtonVariant.Link, size: ButtonSize.Text }, Undo2),
        ],
      },
    ],
  },
  {
    title: 'Finestre del gioco: riprendere, errori, chiusura, avvisi',
    problem:
      'Tutte le finestre del flusso di gioco (G01 «Riprendi», crash, salvataggio rotto, chiusura, E14, modalità TV) e l’azione dei toast sono solo parole. 20 pulsanti solo testo: 19 icona e testo, 1 resta («Non chiederlo più», una casella).',
    solution:
      'ogni coppia di azioni prende l’icona; «Riprendi» il triangolo pieno. Domanda aperta: l’«Annulla» / «Chiudi» / «Non ora» di una finestra prende la X o resta solo testo per far risaltare l’azione principale?',
    rows: [
      {
        buttons: [
          withIcon({ text: 'Ricomincia dal salvataggio del gioco', size: M }, RotateCcw),
          withIcon({ text: 'Riprendi', variant: P, size: M }, Play, true),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Copia i dettagli per una segnalazione', size: M }, ClipboardCopy),
          withIcon({ text: 'Torna alla libreria', size: M }, Library),
          withIcon(
            { text: 'Riprendi dal salvataggio automatico', variant: P, size: M },
            Play,
            true,
          ),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Chiudi senza salvare', variant: ButtonVariant.Danger, size: M }, X),
          withIcon({ text: 'Aspetta', variant: P, size: M }, Hourglass),
          withIcon(
            { text: 'Annulla', variant: ButtonVariant.RaisedAccent, size: ButtonSize.Toast },
            Undo2,
          ),
        ],
      },
    ],
  },
  {
    title: 'Impostazioni: azioni delle righe, cartelle, scelte multiple',
    problem:
      'Le azioni delle righe («Rimappa i pulsanti», «Controlla ora», «Scegli un file…»…) sono parole; ogni cartella ripete «Togli»; «Scegli» / «Fatto» delle scelte multiple sono parole in accento. 13 pulsanti solo testo: 6 icona e testo, 3 solo icona, 4 restano (l’elenco delle pagine, i colori).',
    solution:
      'l’azione di una riga prende un’icona scelta dalla chiave della riga (una piccola tabella in components/settings); nelle righe ripetute la sola icona.',
    rows: [
      {
        buttons: [
          withIcon({ text: 'Rimappa i pulsanti' }, Gamepad2),
          withIcon({ text: 'Controlla ora' }, RefreshCw),
          withIcon({ text: 'Scegli un file…' }, FileIcon),
          withIcon({ text: 'Aggiungi una cartella…' }, FolderPlus),
        ],
      },
      {
        context: 'D:\\Giochi\\GBA',
        buttons: [
          iconOnly(
            { text: 'Togli', variant: ButtonVariant.GhostRegular, size: ButtonSize.Small },
            X,
            ButtonSize.IconRow,
          ),
        ],
      },
      {
        context: 'Formati dei file: .gba, .gb, .gbc',
        buttons: [
          iconOnly(
            { text: 'Scegli', variant: ButtonVariant.GhostAccent, size: ButtonSize.Small },
            Pencil,
            ButtonSize.IconRow,
          ),
          iconOnly(
            { text: 'Fatto', variant: ButtonVariant.GhostAccent, size: ButtonSize.Small },
            Check,
            ButtonSize.IconRow,
          ),
        ],
      },
    ],
  },
  {
    title: 'Cloud: accesso, stato, giochi su Drive, G03 / G04 / G05',
    problem:
      'L’accesso, «Sincronizza ora», «Esci» e le finestre G03–G05 sono parole; ogni gioco su Drive ripete «Carica» / «Scarica» / «Rimuovi da Drive». 11 pulsanti solo testo: 8 icona e testo, 1 solo icona, 2 restano (un link, la copertina cliccabile).',
    solution:
      'icona e testo (nessun marchio Google: solo l’icona di accesso di Lucide); nelle righe dei giochi la sola icona.',
    rows: [
      {
        buttons: [
          withIcon({ text: 'Accedi con Google', variant: P, size: ButtonSize.Large }, LogIn),
          withIcon({ text: 'Sincronizza ora', size: ButtonSize.Tall }, RefreshCw),
          withIcon({ text: 'Esci', variant: ButtonVariant.Link, size: ButtonSize.Text }, LogOut),
        ],
      },
      {
        context: 'Metroid Fusion · GBA · 8 MB',
        buttons: [
          iconOnly({ text: 'Carica' }, CloudUpload, ButtonSize.ToolIcon),
          iconOnly({ text: 'Rimuovi da Drive' }, Trash2, ButtonSize.ToolIcon),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Annulla', size: M }, X),
          withIcon({ text: 'Gioca comunque', variant: P, size: M }, Play, true),
        ],
      },
    ],
  },
  {
    title: 'Controlli: rimappatura e scorciatoie',
    problem:
      'L’avviso di conflitto, il piede della rimappatura e «Vedi i tasti della tastiera» sono solo parole. 7 pulsanti solo testo, tutti icona e testo.',
    solution: 'icona e testo.',
    rows: [
      {
        buttons: [
          withIcon({ text: 'Scambia' }, ArrowLeftRight),
          withIcon({ text: 'Annulla' }, Undo2),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Ripristina i pulsanti predefiniti' }, RotateCcw),
          withIcon({ text: 'Copia da un altro controller…' }, Copy),
          withIcon({ text: 'Pressione ripetuta (turbo)…' }, Zap),
          withIcon({ text: 'Vedi i tasti della tastiera', size: M }, Keyboard),
        ],
      },
    ],
  },
  {
    title: 'Primo avvio: piede dei passi, cartelle, importazione',
    problem:
      '«Indietro», «Salta» e «Continua» sono solo parole, sul telefono come sul desktop; ogni cartella ripete «Rimuovi». 11 pulsanti solo testo: 10 icona e testo, 1 solo icona.',
    solution:
      'icona e testo nel piede (la freccia di «Continua» davanti alle parole: va bene davanti o la preferisci dopo?), la sola X nelle righe.',
    rows: [
      {
        phone: true,
        buttons: [
          withIcon({ text: 'Indietro', size: ButtonSize.Footer }, ChevronLeft),
          withIcon({ text: 'Salta', size: ButtonSize.Footer }, SkipForward),
          withIcon({ text: 'Continua', variant: P, size: ButtonSize.Large }, ArrowRight),
        ],
      },
      {
        context: 'D:\\Giochi',
        buttons: [
          iconOnly(
            { text: 'Rimuovi', variant: ButtonVariant.GhostRegular, size: ButtonSize.Small },
            X,
            ButtonSize.IconRow,
          ),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Aggiungi una cartella…', size: ButtonSize.Tall }, FolderPlus),
          withIcon({ text: 'Importa i salvataggi' }, Import),
          withIcon({ text: 'Scegli una cartella…' }, FolderOpen),
        ],
      },
    ],
  },
  {
    title: 'Copertina (L12): scelta, screenshot, «Adatta la copertina»',
    problem:
      'I tre piedi del selettore di copertine sono solo parole. 6 pulsanti, tutti icona e testo.',
    solution: 'icona e testo.',
    rows: [
      {
        buttons: [
          withIcon({ text: 'Annulla', size: M }, X),
          withIcon({ text: 'Continua', variant: P, size: M }, ArrowRight),
        ],
      },
      {
        buttons: [
          withIcon({ text: 'Indietro', size: M }, ChevronLeft),
          withIcon({ text: 'Usa questa copertina', variant: P, size: M }, Check),
        ],
      },
    ],
  },
  {
    title: 'Editor degli schermi (C09) e dei comandi touch (C06)',
    problem:
      'Nel pannello di C09 e nella barra di C06 «Riduci», «Annulla», «Salva» e gli strumenti sono parole accanto ad «Annulla» / «Ripeti» che sono già icone. 13 pulsanti solo testo: 9 icona e testo, 4 restano (i layout iniziali e i nomi dei comandi), più i 7 strumenti di C06.',
    solution:
      'gli strumenti di C06 come icone sole (la barra ci sta meglio e «Altro» si apre meno spesso), «Salva» e gli altri con icona e testo.',
    rows: [
      {
        buttons: [
          withIcon(
            { text: 'Riduci', variant: ButtonVariant.Ghost, size: ButtonSize.Small },
            ChevronDown,
          ),
          withIcon({ text: 'Annulla', variant: ButtonVariant.Ghost }, X),
          withIcon({ text: 'Salva', variant: P }, Save),
        ],
      },
      {
        phone: true,
        buttons: [
          iconOnly({ text: 'Aggiungi', size: ButtonSize.Tool }, Plus, ButtonSize.ToolIcon),
          iconOnly({ text: 'Dimensione', size: ButtonSize.Tool }, Scaling, ButtonSize.ToolIcon),
          iconOnly({ text: 'Trasparenza', size: ButtonSize.Tool }, Blend, ButtonSize.ToolIcon),
          iconOnly({ text: 'Comportamento', size: ButtonSize.Tool }, Repeat2, ButtonSize.ToolIcon),
          withIcon({ text: 'Salva', variant: P, size: ButtonSize.Tool }, Save),
        ],
      },
    ],
  },
  {
    title: 'Shell: aggiornamento, spazio, notifiche',
    problem:
      'La scheda dell’aggiornamento, l’avviso dello spazio (telefono) e «Segna tutte come lette» sono parole. 6 pulsanti solo testo: 3 icona e testo, 1 solo icona, 2 restano (le notifiche stesse, le schede della TV).',
    solution:
      'icona e testo; «Segna tutte come lette» diventa l’icona sola nella testata del pannello. Domanda aperta: le schede TV prendono le icone della barra laterale?',
    rows: [
      {
        buttons: [
          withIcon(
            { text: 'Più tardi', variant: ButtonVariant.GhostRegular, size: ButtonSize.Small },
            Clock,
          ),
          withIcon({ text: 'Riavvia e aggiorna', variant: P, size: ButtonSize.Small }, RefreshCw),
          iconOnly(
            { text: 'Segna tutte come lette', variant: ButtonVariant.Link, size: ButtonSize.Text },
            CheckCheck,
            ButtonSize.IconSmall,
            { variant: ButtonVariant.Ghost },
          ),
        ],
      },
      {
        phone: true,
        buttons: [withIcon({ text: 'Gestisci lo spazio' }, HardDrive)],
      },
    ],
  },
  {
    title: 'Restano testo: link, scelte, schede',
    problem:
      'Non tutto prende un’icona: i link dentro o accanto a una frase, le scelte in cui le parole sono il valore (ordinamento, «Pixel nitidi», i layout degli schermi), le schede della TV, l’elenco delle pagine delle impostazioni. 26 pulsanti in tutto.',
    solution: 'restano come sono. Domanda aperta: i link prendono una freccia › finale?',
    rows: [
      {
        buttons: [
          same({
            text: 'Altri slot e salvataggi',
            variant: ButtonVariant.Link,
            size: ButtonSize.LabelLink,
          }),
          same({
            text: 'Imposta le velocità',
            variant: ButtonVariant.Link,
            size: ButtonSize.LabelLink,
          }),
          same({ text: 'Pixel nitidi', variant: ButtonVariant.Pick, size: ButtonSize.MenuCompact }),
        ],
      },
    ],
  },
]
