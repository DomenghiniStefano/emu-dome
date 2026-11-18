<script setup lang="ts">
// «…» on a cover (board L01, «Altre azioni»; #59 item 2): the game's actions that have a backend
// today. «Scegli la copertina…» opens L12 (it moves to Game details, L04, when that exists:
// Cowork, #40), then favourite and hide (`library.setFlags`).
// Controller / keyboard: its own nav scope while open; B / Esc / a click outside close it.
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { EyeOff, Eye, ImageIcon, Star, StarOff } from '@lucide/vue'
import { useNavScope } from '../nav'
import type { LibraryGame } from '../emu'

const props = defineProps<{ game: LibraryGame }>()
const emit = defineEmits<{
  cover: []
  flags: [{ favourite?: boolean; hidden?: boolean }]
  close: []
}>()

const { t } = useI18n()

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true, back: () => emit('close') })

function onKey(e: KeyboardEvent) {
  if (e.code === 'Escape') emit('close')
}
// A click outside closes it; the «…» button next to it toggles it itself.
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
  <div
    ref="root"
    class="menu glass-thick"
    role="menu"
    :aria-label="t('library.moreFor', { title: props.game.title })"
  >
    <button class="item" role="menuitem" data-nav-initial @click="emit('cover')">
      <ImageIcon :size="16" :stroke-width="1.75" aria-hidden="true" />
      {{ t('library.menu.cover') }}
    </button>
    <button
      class="item"
      role="menuitem"
      @click="emit('flags', { favourite: !props.game.favourite })"
    >
      <component
        :is="props.game.favourite ? StarOff : Star"
        :size="16"
        :stroke-width="1.75"
        aria-hidden="true"
      />
      {{ t(props.game.favourite ? 'library.menu.unfavourite' : 'library.menu.favourite') }}
    </button>
    <button class="item" role="menuitem" @click="emit('flags', { hidden: !props.game.hidden })">
      <component
        :is="props.game.hidden ? Eye : EyeOff"
        :size="16"
        :stroke-width="1.75"
        aria-hidden="true"
      />
      {{ t(props.game.hidden ? 'library.menu.unhide' : 'library.menu.hide') }}
    </button>
  </div>
</template>

<style scoped>
.menu {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  z-index: 10;
  min-width: 220px;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.item {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  padding: 0 10px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  color: var(--ink);
  text-align: left;
  white-space: nowrap;
}
.item:hover {
  background: var(--surface-hover);
}
.item svg {
  color: var(--ink-muted);
}
</style>
