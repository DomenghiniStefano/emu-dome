/** What a toast says: a notice, or an error (in danger ink, and longer outside the game). */
export const ToastKind = {
  Info: 'info',
  Error: 'error',
} as const
export type ToastKind = (typeof ToastKind)[keyof typeof ToastKind]

/** How long a toast stays, in ms: as asked, else 2 s, and 5 s for an error outside the game
 * (in game, G13: 2 s at most for every kind). */
export function toastDuration(kind: ToastKind, inGame: boolean, asked?: number): number {
  if (asked !== undefined) return asked
  return kind === ToastKind.Error && !inGame ? 5000 : 2000
}

/** A button on a toast (G17 «Caricato lo slot N · Annulla»): it closes the toast when pressed. */
export interface ToastAction {
  label: string
  run: () => void
}
export interface ToastOptions {
  /** A second, quieter part of the text (G17 with a controller: «Annulla dal menu rapido»). */
  note?: string
  action?: ToastAction
  /** How long it stays, in ms (the default: `toastDuration`). */
  ms?: number
}
