<script setup lang="ts">
// Filter and sort panel (board L03, Stati): the order comes from the `library.sort` setting
// (the parent reads and writes it), the «Mostra» switches are the library view's own state.
// Controller / keyboard: its own nav scope while open; B / Esc / a click outside close it.
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useNavScope } from '../nav'
import { SHOW_KEYS, SORTS, type ShowKey, type Sort } from '../librarySort'

defineProps<{ sort: Sort; show: Record<ShowKey, boolean> }>()
const emit = defineEmits<{ sort: [Sort]; toggle: [ShowKey]; reset: []; close: [] }>()

const { t } = useI18n()

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true, back: () => emit('close') })

function onKey(e: KeyboardEvent) {
  if (e.code === 'Escape') emit('close')
}
// A click outside closes it; the parent holds the panel next to its toggle button, which
// toggles it itself (closing here too would reopen it on the same click).
function onPointer(e: PointerEvent) {
  const area = root.value?.parentElement
  if (area && e.target instanceof Node && !area.contains(e.target)) emit('close')
}
onMounted(() => {
  window.addEventListener('keydown', onKey)
  window.addEventListener('pointerdown', onPointer, { capture: true })
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey)
  window.removeEventListener('pointerdown', onPointer, { capture: true })
})
</script>

<template>
  <div ref="root" class="panel glass-thick" role="dialog" :aria-label="t('library.filters.label')">
    <div class="overline">{{ t('library.filters.sortBy') }}</div>
    <div class="options" role="radiogroup" :aria-label="t('library.filters.sortBy')">
      <button
        v-for="s in SORTS"
        :key="s"
        class="option"
        :class="{ on: s === sort }"
        role="radio"
        :aria-checked="s === sort"
        :data-nav-initial="s === sort ? '' : undefined"
        @click="emit('sort', s)"
      >
        <span class="radio" aria-hidden="true"></span>
        {{ t(`library.sort.${s}`) }}
      </button>
    </div>
    <div class="overline">{{ t('library.filters.show') }}</div>
    <div class="options">
      <button
        v-for="k in SHOW_KEYS"
        :key="k"
        class="option"
        role="checkbox"
        :aria-checked="show[k]"
        @click="emit('toggle', k)"
      >
        <span class="check" :class="{ on: show[k] }" aria-hidden="true"></span>
        {{ t(`library.show.${k}`) }}
      </button>
    </div>
    <button class="btn" @click="emit('reset')">{{ t('library.filters.reset') }}</button>
  </div>
</template>

<style scoped>
.panel {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  z-index: 10;
  width: 320px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.options {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.option {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 28px;
  padding: 0 6px;
  margin: 0 -6px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  text-align: left;
}
.option:hover {
  background: var(--surface-hover);
}
.option.on {
  color: var(--accent-text);
  font-weight: 600;
}
.radio,
.check {
  flex: 0 0 14px;
  height: 14px;
  box-sizing: border-box;
  border: 1px solid var(--line-strong);
}
.radio {
  border-radius: 999px;
}
.option.on .radio {
  border: 4px solid var(--accent-text);
}
.check {
  border-radius: 4px;
}
.check.on {
  border: 0;
  background: var(--accent-text);
}
</style>
