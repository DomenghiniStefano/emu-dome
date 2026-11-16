<script setup lang="ts">
import { ref } from 'vue'

interface Toast {
  id: number
  text: string
  kind: 'info' | 'error'
}
const toasts = ref<Toast[]>([])
let next = 0

/** Short message at the bottom left, already translated; it goes away by itself (errors stay longer). */
function push(text: string, kind: Toast['kind'] = 'info') {
  const id = next++
  toasts.value.push({ id, text, kind })
  window.setTimeout(
    () => (toasts.value = toasts.value.filter(t => t.id !== id)),
    kind === 'error' ? 5000 : 2000,
  )
}

defineExpose({ push })
</script>

<template>
  <div class="toasts" aria-live="polite">
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
