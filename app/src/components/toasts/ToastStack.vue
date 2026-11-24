<script setup lang="ts">
// The short messages (G13, G17): bottom left in the shell (on the phone centred above the tab
// bar, #142); in game at the top, never over the
// centre of the game. The list is the app's toasts (`composables/useToasts.ts`); each toast goes
// away by itself, its time counted from when the stack shows it (errors stay longer outside the
// game), so one pushed before the stack was mounted still gets its full time.
import { computed, onBeforeUnmount, watch } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { useToasts } from '@/composables/useToasts'
import { cn } from '@/design-system/lib/cn'
import { ToastKind, toastAreaStyle, toastDuration, type Toast, type ToastArea } from './toasts'

const props = defineProps<{
  inGame?: boolean
  /** The phone's shell: the sections' room the stack sits in (#142); none on the desktop. */
  area?: ToastArea | null
}>()
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
// as wide as its own text (`items-start` / `items-center` on the stack). In the shell it keeps
// above Android's navigation bar, which the shell shows (#131). The phone's shell places it in
// the room its sections get (#142): centred inside the side gutters, above the tab bar, beside
// the side column.
const inArea = computed(() => (!props.inGame && props.area ? props.area : null))
const stackClass = computed(() => {
  if (props.inGame) return 'top-12 left-1/2 -translate-x-1/2 items-center'
  if (inArea.value) return 'items-center px-section-x'
  return 'bottom-[calc(var(--spacing-12)+var(--spacing-safe-bottom))] left-rail ml-12 items-start tv:left-0'
})
const stackStyle = computed(() => (inArea.value ? toastAreaStyle(inArea.value) : undefined))
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
    :style="stackStyle"
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
        :icon="t.action.icon"
        @click="act(t)"
      >
        {{ t.action.label }}
      </Button>
    </div>
  </div>
</template>
