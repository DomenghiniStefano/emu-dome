<script setup lang="ts">
// A count and what it counts, on a snug card (the first run's S02 games per console, the
// library's L11 games a scan added). The number in the stat type (`stat-md`, pending design),
// with tabular figures so a count that grows does not shift.
import type { HTMLAttributes } from 'vue'
import { Card, CardPad } from '@/design-system/components/card'

const props = defineProps<{
  value: number | string
  label: string
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <Card data-slot="stat-tile" :pad="CardPad.Snug" :class="props.class">
    <span class="text-stat-md tabular-nums">{{ value }}</span>
    <span class="text-caption text-ink-muted">{{ label }}</span>
  </Card>
</template>
