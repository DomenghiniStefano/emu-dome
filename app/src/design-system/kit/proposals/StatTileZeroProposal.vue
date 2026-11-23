<script setup lang="ts">
// Kit proposal (StatTile): the tile as it is, with a count of zero in faint ink, so the consoles
// that have games stand out among the four of the first run. Shown on the Kit page only; if the
// owner takes it, StatTile.vue itself dims its zero.
import { Card, CardPad } from '@/design-system/components/card'

defineProps<{ value: number; label: string }>()
</script>

<template>
  <Card :pad="CardPad.Snug">
    <span class="text-stat-md tabular-nums" :class="{ 'text-ink-faint': value === 0 }">
      {{ value }}
    </span>
    <span class="text-caption text-ink-muted">{{ label }}</span>
  </Card>
</template>
