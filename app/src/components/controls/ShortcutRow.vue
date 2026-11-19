<script setup lang="ts">
// One row of C03's shortcut list: the action, what to press (the default slot) with its badge
// when reserved, and the row's own control (the `change` slot: «Cambia», or nothing).
import { Badge, BadgeVariant } from '@/components/ui/badge'
defineProps<{
  action: string
  /** A reserved or keyboard-only row: the action in muted ink. */
  muted?: boolean
  /** «In arrivo», «Con Nintendo DS»: a reserved row's badge. */
  badge?: string
}>()
</script>

<template>
  <div
    class="grid grid-cols-[minmax(0,1fr)_auto_var(--spacing-shortcut-col)] items-center gap-12 border-line px-16 py-10 not-first:border-t"
  >
    <span :class="{ 'text-ink-muted': muted }">{{ action }}</span>
    <span
      class="inline-flex min-w-shortcut-key items-center justify-end gap-8 text-right font-semibold tabular-nums"
    >
      <Badge v-if="badge" :variant="BadgeVariant.Reserved">{{ badge }}</Badge>
      <slot />
    </span>
    <slot name="change"><span></span></slot>
  </div>
</template>
