<script setup lang="ts">
// The tab row. `loop` is off: past the last tab the step leaves the row (nav.ts moves on) instead
// of Reka wrapping to the first. `shrink-0`: a screen puts the row in a flex column (its Tabs root
// `contents`), where an item shrinks to its minimum height when the column overflows; a row that
// scrolls sideways (`overflow-x-auto`) has a minimum of zero, so it collapsed to its scrollbar as
// soon as the page scrolled (Trello #64).
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
    :class="cn('flex shrink-0 gap-4 border-b border-line', props.class)"
  >
    <slot />
  </TabsList>
</template>
