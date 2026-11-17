<script setup lang="ts">
// The open menu of a SettingSelect: its own component so its nav scope exists exactly while it
// is shown (the current option takes the focus, B closes).
import { ref } from 'vue'
import { useNavScope } from '../nav'

defineProps<{ label: string; value: string; options: { id: string; text: string }[] }>()
const emit = defineEmits<{ pick: [string]; close: [] }>()

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true, back: () => emit('close') })
</script>

<template>
  <div ref="root" class="menu glass-thick" role="listbox" :aria-label="label">
    <button
      v-for="o in options"
      :key="o.id"
      class="option"
      :class="{ on: o.id === value }"
      role="option"
      :aria-selected="o.id === value"
      :data-nav-initial="o.id === value ? '' : undefined"
      @click="emit('pick', o.id)"
    >
      {{ o.text }}
    </button>
  </div>
</template>

<style scoped>
.menu {
  position: absolute;
  z-index: 20;
  right: 0;
  top: calc(100% + 4px);
  min-width: 100%;
  max-height: 320px;
  overflow-y: auto;
  padding: 4px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.option {
  height: 32px;
  padding: 0 10px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  text-align: left;
  white-space: nowrap;
}
.option:hover {
  background: var(--surface-hover);
}
.option.on {
  background: var(--accent-soft);
  color: var(--accent-text);
  font-weight: 600;
}
</style>
