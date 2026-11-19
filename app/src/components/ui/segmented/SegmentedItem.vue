<script setup lang="ts">
import type { ToggleGroupItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed, inject } from 'vue'
import { ToggleGroupItem } from 'reka-ui'
import { cn } from '@/lib/cn'
import { SegmentedVariantKey, segmentedItemVariants } from './variants'

const props = defineProps<ToggleGroupItemProps & { class?: HTMLAttributes['class'] }>()
const variant = inject(SegmentedVariantKey, undefined)
const classes = computed(() => cn(segmentedItemVariants({ variant: variant?.value }), props.class))

const delegated = computed(() => {
  const { class: _class, ...rest } = props
  return rest
})
</script>

<template>
  <ToggleGroupItem data-slot="segmented-item" v-bind="delegated" :class="classes">
    <slot />
  </ToggleGroupItem>
</template>
