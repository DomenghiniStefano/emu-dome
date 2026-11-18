<script setup lang="ts">
// A row that leads to another page (Schermi e immagine, Controlli, the backups): its icon, its
// name and a chevron.
import type { Component } from 'vue'
import { ChevronRight } from '@lucide/vue'
import { Card, CardVariant } from '@/components/ui/card'

defineProps<{ icon: Component; label: string; disabled?: boolean }>()
defineEmits<{ click: [Event] }>()
</script>

<template>
  <Card
    as="button"
    :variant="CardVariant.Row"
    :disabled="disabled"
    class="min-w-0 gap-12"
    @click="$emit('click', $event)"
  >
    <span
      class="grid size-32 shrink-0 place-items-center rounded-[8px] bg-surface-raised text-ink-muted"
      aria-hidden="true"
    >
      <component :is="icon" :size="18" :stroke-width="1.75" />
    </span>
    <span class="min-w-0 flex-1 font-semibold">{{ label }}</span>
    <ChevronRight :size="16" :stroke-width="1.75" class="shrink-0 text-ink-muted" aria-hidden="true" />
  </Card>
</template>
