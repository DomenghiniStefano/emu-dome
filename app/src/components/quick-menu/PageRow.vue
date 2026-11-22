<script setup lang="ts">
// A row that leads to another page (Schermi e immagine, Controlli, the backups): its icon, its
// name and a chevron. On the phone's «Altro» (owner, 2026-11-22) also a value before the chevron
// («Slot rapido: 1»), and rows that act instead of leading (no chevron), some staying on
// (`pressed`: «Scambia gli schermi», the lid).
import type { Component } from 'vue'
import { ChevronRight } from '@lucide/vue'
import { Card, CardVariant } from '@/components/ui/card'

withDefaults(
  defineProps<{
    icon: Component
    label: string
    disabled?: boolean
    /** A value shown before the chevron. */
    value?: string
    /** False: a row that acts here, no chevron. */
    leads?: boolean
    /** A row that stays on: whether it is on now (`aria-pressed`). */
    pressed?: boolean
  }>(),
  { leads: true, value: undefined, pressed: undefined },
)
defineEmits<{ click: [Event] }>()
</script>

<template>
  <Card
    as="button"
    :variant="CardVariant.NavRow"
    :disabled="disabled"
    :selected="pressed"
    :aria-pressed="pressed"
    class="min-w-0 gap-8"
    @click="$emit('click', $event)"
  >
    <span
      class="grid size-control-sm shrink-0 place-items-center rounded-control bg-surface-raised text-ink-muted"
      aria-hidden="true"
    >
      <component :is="icon" :size="18" :stroke-width="1.75" />
    </span>
    <span class="min-w-0 flex-1 truncate font-semibold">{{ label }}</span>
    <span v-if="value" class="shrink-0 text-caption text-ink-muted tabular-nums">{{ value }}</span>
    <ChevronRight
      v-if="leads"
      :size="16"
      :stroke-width="1.75"
      class="shrink-0 text-ink-muted"
      aria-hidden="true"
    />
  </Card>
</template>
