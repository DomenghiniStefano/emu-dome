// The toasts: what the shell and the game tell the user (G13, G17). One list for the whole app,
// created by App.vue and handed down with provide / inject, so a screen tells the user something
// itself instead of relaying it up through `error` / `notice` emits.
//
// The list lives here, not in ToastStack: a toast pushed before the stack is mounted waits in the
// list, and shows (its time starting) once the stack is there.
import { inject, provide, ref, type InjectionKey, type Ref } from 'vue'
import { errorText, type ErrorSink } from '@/i18n/labels'
import { reportError } from '@/lib/errors'
import { ToastKind, type Toast, type ToastOptions } from '@/components/toasts/toasts'

export interface Toasts {
  /** The toasts shown now, oldest first (ToastStack renders them). */
  readonly list: Readonly<Ref<readonly Toast[]>>
  /** A notice, already translated. */
  notify: (text: string, opts?: ToastOptions) => void
  /** An error, already translated. */
  fail: (text: string) => void
  /** An error from the bridge (an `ErrorCode`), translated here. */
  failWith: (e: unknown) => void
  /** Takes a toast away (its time is up, or its action was pressed). */
  dismiss: (id: number) => void
}

export function createToasts(): Toasts {
  const list = ref<Toast[]>([])
  let next = 0
  const push = (text: string, kind: ToastKind, opts: ToastOptions = {}) => {
    list.value = [...list.value, { ...opts, id: next++, text, kind }]
  }
  return {
    list,
    notify: (text, opts) => push(text, ToastKind.Info, opts),
    fail: text => push(text, ToastKind.Error),
    failWith: e => push(errorText(e), ToastKind.Error),
    dismiss: id => (list.value = list.value.filter(t => t.id !== id)),
  }
}

const TOASTS: InjectionKey<Toasts> = Symbol('toasts')

/** App.vue: the toasts every component below it reaches with `useToasts` / `useErrorSink`. */
export function provideToasts(toasts: Toasts) {
  provide(TOASTS, toasts)
}

/** The app's toasts (provided by App.vue). */
export function useToasts(): Toasts {
  const toasts = inject(TOASTS, null)
  if (!toasts) throw new Error('useToasts: no toasts provided (App.vue provides them)')
  return toasts
}

/**
 * Where a component sends a failure it does not show itself: logged, and shown as a toast. With no
 * toasts provided (the Kit page, a test) it goes to `reportError`, so it is still logged.
 */
export function useErrorSink(): ErrorSink {
  const toasts = inject(TOASTS, null)
  if (!toasts) return reportError
  return e => {
    console.error(e)
    toasts.failWith(e)
  }
}
