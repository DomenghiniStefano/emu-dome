<script setup lang="ts">
// The phone's main screen under speed: Schermi e immagine, Controlli and the tools in one grid of
// two columns, for the thumb.
import type { Component } from 'vue'
import { useI18n } from 'vue-i18n'
import { Gamepad2, Rows2 } from '@lucide/vue'
import { Card, CardVariant } from '@/components/ui/card'
import type { Tool } from './tools'
import { QuickMenuView, openerMark } from './view'

defineProps<{
  tools: Tool[]
  /** Why the disabled tools cannot be used; '' with none. */
  note: string
}>()
const emit = defineEmits<{
  open: [QuickMenuView, Event]
  run: [Tool, Event]
}>()

const { t } = useI18n()

const PAGES: { view: QuickMenuView; icon: Component; label: string }[] = [
  { view: QuickMenuView.Screens, icon: Rows2, label: 'qm.row.screens.name' },
  { view: QuickMenuView.Controls, icon: Gamepad2, label: 'qm.row.controls.name' },
]
</script>

<template>
  <nav class="grid grid-cols-[1fr_1fr] gap-8" :aria-label="t('qm.toolsTitle')">
    <Card
      v-for="p in PAGES"
      :key="p.view"
      v-bind="openerMark(p.view)"
      as="button"
      :variant="CardVariant.Tool"
      class="min-w-0 gap-8"
      @click="emit('open', p.view, $event)"
    >
      <component
        :is="p.icon"
        :size="18"
        :stroke-width="1.75"
        class="shrink-0 text-ink-muted"
        aria-hidden="true"
      />
      <span class="overflow-hidden text-ellipsis">{{ t(p.label) }}</span>
    </Card>
    <Card
      v-for="x in tools"
      :key="x.id"
      v-bind="openerMark(x.id)"
      as="button"
      :variant="CardVariant.Tool"
      :selected="x.pressed"
      class="min-w-0 gap-8"
      :disabled="!x.available"
      :aria-pressed="x.pressed"
      @click="emit('run', x, $event)"
    >
      <component
        :is="x.icon"
        :size="18"
        :stroke-width="1.75"
        class="shrink-0 text-ink-muted"
        aria-hidden="true"
      />
      <span class="overflow-hidden text-ellipsis">{{ t(x.label) }}</span>
    </Card>
  </nav>
  <span v-if="note" class="text-caption text-ink-muted">{{ note }}</span>
</template>
