<script setup lang="ts">
import { ref } from 'vue'

/** A button on a toast (G17 «Caricato lo slot N · Annulla»): it closes the toast when pressed. */
export interface ToastAction {
  label: string
  run: () => void
}
export interface ToastOptions {
  /** A second, quieter part of the text (G17 with a controller: «Annulla dal menu rapido»). */
  note?: string
  action?: ToastAction
  /** How long it stays, in ms (the default: 2 s in game, 5 s for errors outside it). */
  ms?: number
}

interface Toast {
  id: number
  text: string
  kind: 'info' | 'error'
  note?: string
  action?: ToastAction
}
/** In game (G13): at the top, never over the centre of the game, 2 s at most for every kind. */
const props = defineProps<{ inGame?: boolean }>()
const toasts = ref<Toast[]>([])
let next = 0

const remove = (id: number) => (toasts.value = toasts.value.filter(t => t.id !== id))

/** Short message at the bottom left, already translated; it goes away by itself (errors stay longer). */
function push(text: string, kind: Toast['kind'] = 'info', opts: ToastOptions = {}) {
  const id = next++
  toasts.value.push({ id, text, kind, note: opts.note, action: opts.action })
  window.setTimeout(() => remove(id), opts.ms ?? (kind === 'error' && !props.inGame ? 5000 : 2000))
}

function act(t: Toast) {
  remove(t.id)
  t.action?.run()
}

defineExpose({ push })
</script>

<template>
  <div class="toasts" :class="{ top: inGame }" aria-live="polite">
    <div
      v-for="t in toasts"
      :key="t.id"
      class="toast glass-thick"
      :class="[t.kind, { actionable: t.action }]"
    >
      <span>
        {{ t.text }}
        <span v-if="t.note" class="note">{{ t.note }}</span>
      </span>
      <button v-if="t.action" class="toast-action" @click="act(t)">{{ t.action.label }}</button>
    </div>
  </div>
</template>

<style scoped>
.toasts {
  position: fixed;
  left: 16px;
  bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
  z-index: 40;
}
.toasts.top {
  left: 50%;
  bottom: auto;
  top: 16px;
  transform: translateX(-50%);
  align-items: center;
}
.toast {
  padding: 10px 14px;
  border-radius: var(--radius-10);
  font-weight: 500;
  max-width: 420px;
}
.toast.error {
  color: var(--danger);
}
.note {
  color: var(--ink-muted);
  font-weight: 400;
}
.note::before {
  content: ' · ';
}
/* G17: «Caricato lo slot N» with its «Annulla» button, 44 px tall. */
.toast.actionable {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 44px;
  padding: 6px 6px 6px 16px;
  border-radius: 12px;
  font-weight: 600;
  pointer-events: auto;
  white-space: nowrap;
}
.toast-action {
  min-height: 32px;
  padding: 0 10px;
  border: 0;
  border-radius: 8px;
  background: var(--surface-raised);
  color: var(--accent-text);
  font-weight: 600;
}
.toast-action:hover {
  background: var(--surface-hover);
}
@media (pointer: coarse) {
  .toast-action {
    min-height: 44px;
    padding: 0 16px;
  }
}
</style>
