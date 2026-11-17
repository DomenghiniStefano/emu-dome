<script setup lang="ts">
import { ref } from 'vue'

interface Toast {
  id: number
  text: string
  kind: 'info' | 'error'
}
/** In game (G13): at the top, never over the centre of the game, 2 s at most for every kind. */
const props = defineProps<{ inGame?: boolean }>()
const toasts = ref<Toast[]>([])
let next = 0

/** Short message at the bottom left, already translated; it goes away by itself (errors stay longer). */
function push(text: string, kind: Toast['kind'] = 'info') {
  const id = next++
  toasts.value.push({ id, text, kind })
  window.setTimeout(
    () => (toasts.value = toasts.value.filter(t => t.id !== id)),
    kind === 'error' && !props.inGame ? 5000 : 2000,
  )
}

defineExpose({ push })
</script>

<template>
  <div class="toasts" :class="{ top: inGame }" aria-live="polite">
    <div v-for="t in toasts" :key="t.id" class="toast glass-thick" :class="t.kind">
      {{ t.text }}
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
</style>
