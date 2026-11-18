<script setup lang="ts">
import type { ToggleGroupItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { ToggleGroupItem } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<ToggleGroupItemProps & { class?: HTMLAttributes['class'] }>()

const delegated = computed(() => {
  const { class: _class, ...rest } = props
  return rest
})
</script>

<template>
  <ToggleGroupItem
    data-slot="segmented-item"
    v-bind="delegated"
    :class="
      cn(
        'h-control-sm cursor-pointer rounded-4 border-0 bg-transparent px-10 font-sans text-label text-ink-muted disabled:cursor-not-allowed data-[state=on]:bg-accent-soft data-[state=on]:font-semibold data-[state=on]:text-accent-text',
        props.class,
      )
    "
  >
    <slot />
  </ToggleGroupItem>
</template>
