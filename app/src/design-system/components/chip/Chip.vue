<script setup lang="ts">
// A chip: a filter or an option that is on or off. A native <button> by default, so nav.ts moves
// to it and clicks it on `confirm`; `as="span"` for a chip that only shows a value.
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import type { ChipVariant } from './variants'
import { chipVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: ChipVariant
  selected?: boolean
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), { as: 'button' })
</script>

<template>
  <Primitive
    data-slot="chip"
    :as="as"
    :as-child="asChild"
    :type="as === 'button' && !asChild ? 'button' : undefined"
    :data-selected="selected ? '' : undefined"
    :class="cn(chipVariants({ variant }), props.class)"
  >
    <slot />
  </Primitive>
</template>
