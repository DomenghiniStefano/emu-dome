<script setup lang="ts">
// The short messages (G13, G17): bottom left in the shell; in game at the top, never over the
// centre of the game. Each goes away by itself; App.vue pushes them through `push`.
import { computed, ref } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { cn } from '@/lib/cn'
import { ToastKind, toastDuration, type ToastAction, type ToastOptions } from '../lib/shell/toasts'

interface Toast {
  id: number
  text: string
  kind: ToastKind
  note?: string
  action?: ToastAction
}
const props = defineProps<{ inGame?: boolean }>()
const toasts = ref<Toast[]>([])
let next = 0

const remove = (id: number) => (toasts.value = toasts.value.filter(t => t.id !== id))

/** A message already translated; it goes away by itself (errors stay longer outside the game). */
function push(text: string, kind: ToastKind = ToastKind.Info, opts: ToastOptions = {}) {
  const id = next++
  toasts.value.push({ id, text, kind, note: opts.note, action: opts.action })
  window.setTimeout(() => remove(id), toastDuration(kind, !!props.inGame, opts.ms))
}

function act(t: Toast) {
  remove(t.id)
  t.action?.run()
}

defineExpose({ push })

// The radius 12 and the 44px row of a toast with an action wait for Cowork (spec Decision 5).
const stackClass = computed(() =>
  props.inGame ? 'top-16 left-1/2 -translate-x-1/2 items-center' : 'bottom-16 left-16',
)
const toastClass = (t: Toast) =>
  cn(
    'glass-thick max-w-[420px]',
    t.action
      ? 'pointer-events-auto flex min-h-[44px] items-center gap-12 rounded-[12px] py-6 pr-6 pl-16 font-semibold whitespace-nowrap'
      : 'rounded-10 px-14 py-10 font-medium',
    t.kind === ToastKind.Error && 'text-danger',
  )
</script>

<template>
  <div
    :class="['pointer-events-none fixed z-toast flex flex-col gap-8', stackClass]"
    aria-live="polite"
  >
    <div v-for="t in toasts" :key="t.id" :class="toastClass(t)">
      <span>
        {{ t.text }}
        <span v-if="t.note" class="font-regular text-ink-muted before:content-['_·_']">
          {{ t.note }}
        </span>
      </span>
      <Button
        v-if="t.action"
        :variant="ButtonVariant.SoftAccent"
        :size="ButtonSize.Toast"
        @click="act(t)"
      >
        {{ t.action.label }}
      </Button>
    </div>
  </div>
</template>
