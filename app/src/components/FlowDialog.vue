<script setup lang="ts">
// A blocking dialog of the game flow (G02, E11, E12, the resume question): glass panel over a
// scrim, its own nav scope while shown (the `data-nav-initial` action takes the focus; B is
// `back`). Board rule: one action in accent at most, the safe exit always there.
import { ref } from 'vue'
import { useNavScope } from '../nav'

defineProps<{ title: string; tone?: 'danger' | 'warning' }>()
const emit = defineEmits<{ back: [] }>()

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true, back: () => emit('back') })
</script>

<template>
  <div class="scrim">
    <div ref="root" class="dialog glass-thick" role="alertdialog" :aria-label="title">
      <h2 class="title" :class="tone">{{ title }}</h2>
      <div class="body"><slot /></div>
      <div class="actions"><slot name="actions" /></div>
    </div>
  </div>
</template>

<style scoped>
.scrim {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: grid;
  place-items: center;
  padding: 16px;
  background: var(--scrim);
}
.dialog {
  width: 100%;
  max-width: 520px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.title {
  margin: 0;
  font-size: 18px;
  line-height: 24px;
  font-weight: 600;
}
.title.danger {
  color: var(--danger);
}
.title.warning {
  color: var(--warning);
}
.body {
  color: var(--ink-muted);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
.actions :deep(.btn) {
  height: 36px;
}
.actions :deep(.btn-primary) {
  height: 36px;
  font-size: 13px;
  padding: 0 16px;
}
</style>
