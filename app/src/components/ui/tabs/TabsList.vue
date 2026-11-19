<script setup lang="ts">
// The tab row. `loop` is off: past the last tab the step leaves the row (nav.ts moves on) instead
// of Reka wrapping to the first.
import type { TabsListProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { TabsList } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = withDefaults(defineProps<TabsListProps & { class?: HTMLAttributes['class'] }>(), {
  loop: false,
})
</script>

<template>
  <TabsList
    data-slot="tabs-list"
    :loop="props.loop"
    :as="props.as"
    :as-child="props.asChild"
    :class="cn('flex gap-4 border-b border-line', props.class)"
  >
    <slot />
  </TabsList>
</template>
