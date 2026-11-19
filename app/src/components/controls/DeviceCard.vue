<script setup lang="ts">
// A device on C01: its icon, name and state, and «In uso» on the controller the section starts
// on. One of the devices' radio group: choosing it sets what the rest of C01 shows.
import type { Component } from 'vue'
import { Card, CardVariant } from '@/components/ui/card'
import { Badge, BadgeSize, BadgeVariant } from '@/components/ui/badge'

defineProps<{
  icon: Component
  name: string
  detail: string
  chosen: boolean
  /** «In uso», when given. */
  badge?: string
}>()
</script>

<template>
  <Card
    as="button"
    :variant="CardVariant.Choice"
    :selected="chosen"
    role="radio"
    :aria-checked="chosen"
    class="flex-row items-center gap-12"
  >
    <span
      class="grid size-control-lg flex-none place-items-center rounded-10 bg-surface-raised text-success"
      aria-hidden="true"
    >
      <component :is="icon" :size="22" :stroke-width="1.75" />
    </span>
    <span class="flex min-w-0 flex-1 flex-col">
      <span class="truncate font-semibold">{{ name }}</span>
      <span class="text-caption text-ink-muted">{{ detail }}</span>
    </span>
    <Badge v-if="badge" :variant="BadgeVariant.Success" :size="BadgeSize.Pill">{{ badge }}</Badge>
  </Card>
</template>
