<script setup lang="ts">
// A row of C01's list that leads one step down (C02, C04): its name, a line under it and the
// chevron. A button, or `as="div"` for a row that only says where it is done (touch controls).
// No line between the rows: the card's padding alone parts them.
import { ChevronRight } from '@lucide/vue'
import { Card, CardVariant } from '@/components/ui/card'

withDefaults(defineProps<{ name: string; sub: string; as?: 'button' | 'div' }>(), {
  as: 'button',
})
</script>

<template>
  <Card :as="as" :variant="CardVariant.Row" class="min-h-[56px] gap-16">
    <span class="flex min-w-0 flex-1 flex-col">
      <span class="font-medium">{{ name }}</span>
      <span class="text-caption text-ink-muted">{{ sub }}</span>
    </span>
    <ChevronRight
      class="flex-none text-ink-muted"
      :size="16"
      :stroke-width="1.75"
      aria-hidden="true"
    />
  </Card>
</template>
