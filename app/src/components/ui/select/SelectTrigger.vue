<script setup lang="ts">
// The field showing the chosen option, a chevron at the end; its looks are in variants.ts.
import type { SelectTriggerProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { ChevronDown } from '@lucide/vue'
import { SelectIcon, SelectTrigger } from 'reka-ui'
import { cn } from '@/lib/cn'
import type { SelectTriggerVariant } from './variants'
import { selectTriggerVariants } from './variants'

const props = defineProps<
  SelectTriggerProps & { variant?: SelectTriggerVariant; class?: HTMLAttributes['class'] }
>()

const delegated = computed(() => {
  const { class: _class, variant: _variant, ...rest } = props
  return rest
})
</script>

<template>
  <SelectTrigger
    data-slot="select-trigger"
    v-bind="delegated"
    :class="cn(selectTriggerVariants({ variant }), props.class)"
  >
    <span class="flex-1"><slot /></span>
    <SelectIcon as-child>
      <ChevronDown :size="14" :stroke-width="1.75" aria-hidden="true" />
    </SelectIcon>
  </SelectTrigger>
</template>
