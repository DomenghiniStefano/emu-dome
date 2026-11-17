<script setup lang="ts">
// A list choice (`sel` on the Settings board): a button with the current value that opens the
// options in a small glass menu (SettingSelectMenu). B / Esc / a click outside close it and the
// focus returns to the button.
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { ChevronDown } from '@lucide/vue'
import SettingSelectMenu from './SettingSelectMenu.vue'

const props = defineProps<{
  label: string
  value: string
  options: { id: string; text: string }[]
  disabled?: boolean
}>()
const emit = defineEmits<{ pick: [string] }>()

const open = ref(false)
const wrap = ref<HTMLElement | null>(null)
const button = ref<HTMLButtonElement | null>(null)

function close() {
  open.value = false
  void nextTick(() => button.value?.focus())
}
function pick(id: string) {
  emit('pick', id)
  close()
}

function onKey(e: KeyboardEvent) {
  if (e.code !== 'Escape') return
  e.stopPropagation()
  close()
}
function onPointer(e: PointerEvent) {
  if (e.target instanceof Node && !wrap.value?.contains(e.target)) open.value = false
}
function listen(on: boolean) {
  if (on) {
    window.addEventListener('keydown', onKey, { capture: true })
    window.addEventListener('pointerdown', onPointer, { capture: true })
  } else {
    window.removeEventListener('keydown', onKey, { capture: true })
    window.removeEventListener('pointerdown', onPointer, { capture: true })
  }
}
watch(open, listen)
onBeforeUnmount(() => listen(false))

function current(): string {
  return props.options.find(o => o.id === props.value)?.text ?? props.value
}
</script>

<template>
  <div ref="wrap" class="select">
    <button
      ref="button"
      class="field"
      :aria-label="label"
      aria-haspopup="listbox"
      :aria-expanded="open"
      :disabled="disabled"
      @click="open = !open"
    >
      <span class="text">{{ current() }}</span>
      <ChevronDown :size="14" :stroke-width="1.75" aria-hidden="true" />
    </button>
    <SettingSelectMenu
      v-if="open"
      :label="label"
      :value="value"
      :options="options"
      @pick="pick"
      @close="close"
    />
  </div>
</template>

<style scoped>
.select {
  position: relative;
}
.field {
  min-width: 200px;
  height: 32px;
  padding: 0 10px;
  border-radius: var(--radius-6);
  border: 1px solid var(--line);
  background: var(--surface-raised);
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}
.field:hover:not(:disabled) {
  background: var(--surface-hover);
}
.text {
  flex: 1;
  text-align: left;
}
</style>
