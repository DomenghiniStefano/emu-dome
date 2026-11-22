<script setup lang="ts">
// One result of the search everywhere (X01, mockup «Ricerca e notifiche»): an icon or the game's
// thumbnail, the name over where it lives (or why it cannot be used, X05), and on the right a
// setting's value or an action's shortcut. A click, A or Enter chooses it; Ctrl + Enter on a game
// starts it.
import { computed, type Component } from 'vue'
import {
  ArrowDownUp,
  Camera,
  ChevronLeft,
  ChevronRight,
  FastForward,
  FolderOpen,
  Gauge,
  Laptop,
  Maximize,
  Menu,
  Pause,
  Rewind,
  Rows2,
  Save,
  Settings2,
  Snail,
} from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import type { KeyAction } from '@/lib/keyboard'
import type { SearchItem } from './useSearchSources'

const props = defineProps<{ item: SearchItem }>()
const emit = defineEmits<{ choose: [start: boolean] }>()

/** One icon per action; a new shortcut in Rust fails the typecheck here until it has one. */
const ACTION_ICONS: Record<KeyAction, Component> = {
  menu: Menu,
  save: Save,
  load: FolderOpen,
  next_slot: ChevronRight,
  prev_slot: ChevronLeft,
  ff_hold: FastForward,
  ff_toggle: FastForward,
  slow: Snail,
  rewind: Rewind,
  pause: Pause,
  screenshot: Camera,
  fps: Gauge,
  swap: ArrowDownUp,
  layout: Rows2,
  lid: Laptop,
  fullscreen: Maximize,
}

const icon = computed(() => {
  const i = props.item
  if (i.kind === 'action') return ACTION_ICONS[i.id]
  return i.kind === 'setting' ? Settings2 : null
})
const sub = computed(() => {
  const i = props.item
  if (i.kind === 'action') return i.reason ?? ''
  return i.where
})
const side = computed(() => {
  const i = props.item
  if (i.kind === 'action') return i.shortcut ?? ''
  return i.kind === 'setting' ? i.value : ''
})
const disabled = computed(() => props.item.kind === 'action' && props.item.reason !== null)
</script>

<template>
  <Button
    :variant="ButtonVariant.Result"
    :size="ButtonSize.Result"
    :disabled="disabled"
    @click="emit('choose', false)"
    @keydown.enter.ctrl.exact.stop.prevent="emit('choose', true)"
  >
    <GameCover
      v-if="item.kind === 'game'"
      :game="item.game"
      :size="CoverSize.Thumb"
      :badge="false"
      :title="false"
      class="flex-none"
    />
    <component
      :is="icon"
      v-else
      class="flex-none text-ink-muted"
      :size="20"
      :stroke-width="1.75"
      aria-hidden="true"
    />
    <span class="flex min-w-0 flex-col">
      <span class="font-medium">{{ item.name }}</span>
      <span v-if="sub" class="text-caption text-ink-muted">{{ sub }}</span>
    </span>
    <span v-if="side" class="ml-auto max-w-field min-w-0 truncate text-caption text-ink-muted">
      {{ side }}
    </span>
  </Button>
</template>
