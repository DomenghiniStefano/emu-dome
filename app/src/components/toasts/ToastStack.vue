<script setup lang="ts">
// The short messages (G13, G17): bottom left in the shell; in game at the top, never over the
// centre of the game. The list is the app's toasts (`composables/useToasts.ts`); each toast goes
// away by itself, its time counted from when the stack shows it (errors stay longer outside the
// game), so one pushed before the stack was mounted still gets its full time.
import { computed, onBeforeUnmount, watch } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useToasts } from '@/composables/useToasts'
import { cn } from '@/lib/cn'
import { ToastKind, toastDuration, type Toast } from './toasts'

const props = defineProps<{ inGame?: boolean }>()
const toasts = useToasts()

const timers = new Map<number, number>()
watch(
  toasts.list,
  list => {
    for (const t of list) {
      if (timers.has(t.id)) continue
      const ms = toastDuration(t.kind, !!props.inGame, t.ms)
      timers.set(
        t.id,
        window.setTimeout(() => remove(t.id), ms),
      )
    }
  },
  { immediate: true },
)
onBeforeUnmount(() => timers.forEach(id => window.clearTimeout(id)))

function remove(id: number) {
  window.clearTimeout(timers.get(id))
  timers.delete(id)
  toasts.dismiss(id)
}

function act(t: Toast) {
  remove(t.id)
  t.action?.run()
}

// One shape for every toast: a row as tall as the large control (the touch minimum on the
// phone), its text wrapping; one with an action only adds the button at its end. In the shell
// the stack starts past the rail, never over its sections. Each toast is
// as wide as its own text (`items-start` / `items-center` on the stack).
const stackClass = computed(() =>
  props.inGame
    ? 'top-12 left-1/2 -translate-x-1/2 items-center'
    : 'bottom-12 left-rail ml-12 items-start tv:left-0',
)
const toastClass = (t: Toast) =>
  cn(
    'glass-thick flex max-w-toast min-h-control-lg items-center gap-8 rounded-card py-2 pl-10 font-medium touch:min-h-touch',
    // The action's button (28, the touch 44) fills the row but for a hairline: every toast is
    // as tall, with or without one, and a text that wraps keeps 6 from the edges (its own 4).
    t.action ? 'pointer-events-auto pr-2 touch:py-0' : 'pr-10',
    t.kind === ToastKind.Error && 'text-danger',
  )
</script>

<template>
  <div
    :class="['pointer-events-none fixed z-toast flex flex-col gap-8', stackClass]"
    aria-live="polite"
  >
    <div v-for="t in toasts.list.value" :key="t.id" :class="toastClass(t)">
      <span class="min-w-0 flex-1 py-4">
        {{ t.text }}
        <span v-if="t.note" class="font-regular text-ink-muted before:content-['_·_']">
          {{ t.note }}
        </span>
      </span>
      <Button
        v-if="t.action"
        :variant="ButtonVariant.RaisedAccent"
        :size="ButtonSize.Toast"
        @click="act(t)"
      >
        {{ t.action.label }}
      </Button>
    </div>
  </div>
</template>
