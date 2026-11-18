<script setup lang="ts">
import type { TabsContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { TabsContent } from 'reka-ui'

const props = defineProps<TabsContentProps & { class?: HTMLAttributes['class'] }>()

const delegated = computed(() => {
  const { class: _class, ...rest } = props
  return rest
})
</script>

<template>
  <TabsContent data-slot="tabs-content" v-bind="delegated" :class="props.class">
    <slot />
  </TabsContent>
</template>
