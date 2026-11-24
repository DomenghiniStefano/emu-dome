import type { Component } from 'vue'

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

/** The sections' room in the phone's shell (components/shell/useShellArea.ts), in CSS px. */
export interface ToastArea {
  left: number
  right: number
  bottom: number
}

/**
 * Where the stack sits in the phone's shell (#142): across the sections' room, its toasts
 * centred (the class gives the side gutters), 12 above the room's bottom edge: the tab bar
 * upright, the window's bottom on its side, never over the side column.
 */
export function toastAreaStyle(area: ToastArea): Record<string, string> {
  return {
    left: `${area.left}px`,
    right: `${area.right}px`,
    bottom: `calc(${area.bottom}px + var(--spacing-12))`,
  }
}

/** A button on a toast (G17 «Caricato lo slot N · Annulla»): it closes the toast when pressed. */
export interface ToastAction {
  label: string
  /** The Lucide icon drawn before the label (#104: «Annulla» Undo2, «Usa come copertina» Image). */
  icon?: Component
  run: () => void
}
export interface ToastOptions {
  /** A second, quieter part of the text (G17 with a controller: «Annulla dal menu rapido»). */
  note?: string
  action?: ToastAction
  /** How long it stays, in ms (the default: `toastDuration`). */
  ms?: number
}

/** One toast in the list (`composables/useToasts.ts`), already translated. */
export interface Toast extends ToastOptions {
  id: number
  text: string
  kind: ToastKind
}
