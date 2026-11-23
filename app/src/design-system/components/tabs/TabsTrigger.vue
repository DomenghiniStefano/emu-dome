<script setup lang="ts">
import type { TabsTriggerProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed, inject } from 'vue'
import { TabsTrigger } from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import { TabsSizeKey } from './context'
import { tabsTriggerVariants } from './variants'

const props = defineProps<TabsTriggerProps & { class?: HTMLAttributes['class'] }>()
const size = inject(TabsSizeKey, undefined)

const delegated = computed(() => {
  const { class: _class, ...rest } = props
  return rest
})
</script>

<template>
  <TabsTrigger
    data-slot="tabs-trigger"
    v-bind="delegated"
    :class="cn(tabsTriggerVariants({ size }), props.class)"
  >
    <slot />
  </TabsTrigger>
</template>
