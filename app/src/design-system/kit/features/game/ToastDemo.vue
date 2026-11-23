<script setup lang="ts">
// The real ToastStack over its own fixed list (Kit page only): it provides the toasts App.vue
// would, each one asked to stay (so the timers never take it away), and holds the stack, which is
// `fixed` in the app, in a box of its own.
import { ref } from 'vue'
import ToastStack from '@/components/toasts/ToastStack.vue'
import type { Toast } from '@/components/toasts/toasts'
import { provideToasts } from '@/composables/useToasts'

const props = defineProps<{ toasts: Omit<Toast, 'id' | 'ms'>[]; inGame?: boolean }>()

/** As long as a timer can wait (about 23 days): the toasts stay on the page. */
const STAY = 2_000_000_000
const list = ref<Toast[]>(props.toasts.map((t, id) => ({ ...t, id, ms: STAY })))
provideToasts({
  list,
  notify: () => {},
  fail: () => {},
  failWith: () => {},
  dismiss: id => (list.value = list.value.filter(t => t.id !== id)),
})
</script>

<template>
  <ToastStack :in-game="inGame" />
</template>
