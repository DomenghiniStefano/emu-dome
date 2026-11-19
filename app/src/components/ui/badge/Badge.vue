<script setup lang="ts">
// A short status beside a name (BadgeVariant, BadgeSize). Not focusable; its words come from the
// caller, already translated.
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '@/lib/cn'
import type { BadgeSize, BadgeVariant } from './variants'
import { badgeVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: BadgeVariant
  size?: BadgeSize
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), { as: 'span' })
</script>

<template>
  <Primitive
    data-slot="badge"
    :as="as"
    :as-child="asChild"
    :class="cn(badgeVariants({ variant, size }), props.class)"
  >
    <slot />
  </Primitive>
</template>
