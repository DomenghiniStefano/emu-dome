<script setup lang="ts">
// The field showing the chosen option, a chevron at the end; its looks are in variants.ts. The
// field's value stays on one line, cut with an ellipsis when the field is short (the whole name
// is in the open list).
import type { SelectTriggerProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { ChevronDown } from '@lucide/vue'
import { SelectIcon, SelectTrigger } from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import { SelectTriggerVariant, selectTriggerVariants } from './variants'

const props = defineProps<
  SelectTriggerProps & { variant?: SelectTriggerVariant; class?: HTMLAttributes['class'] }
>()

const delegated = computed(() => {
  const { class: _class, variant: _variant, ...rest } = props
  return rest
})
// A choice inside a title grows with its words; only the field cuts a long value.
const valueClass = computed(() =>
  props.variant === SelectTriggerVariant.InTitle ? 'flex-1' : 'min-w-0 flex-1 truncate',
)
</script>

<template>
  <SelectTrigger
    data-slot="select-trigger"
    v-bind="delegated"
    :class="cn(selectTriggerVariants({ variant }), props.class)"
  >
    <span :class="valueClass"><slot /></span>
    <SelectIcon as-child>
      <ChevronDown :size="14" :stroke-width="1.75" aria-hidden="true" />
    </SelectIcon>
  </SelectTrigger>
</template>
