// The toast stack App.vue renders as `ref="toasts"`: what the shell and the game tell the user.
import { useTemplateRef } from 'vue'
import type ToastStack from '../components/ToastStack.vue'
import { errorText } from '../i18n/labels'
import { ToastKind, type ToastOptions } from '../lib/shell/toasts'

export function useToasts() {
  const stack = useTemplateRef<InstanceType<typeof ToastStack>>('toasts')
  return {
    /** A notice, already translated. */
    notify: (text: string, opts?: ToastOptions) => stack.value?.push(text, ToastKind.Info, opts),
    /** An error, already translated. */
    fail: (text: string) => stack.value?.push(text, ToastKind.Error),
    /** An error from the bridge (an `ErrorCode`), translated here. */
    failWith: (e: unknown) => stack.value?.push(errorText(e), ToastKind.Error),
  }
}
export type Toasts = ReturnType<typeof useToasts>
