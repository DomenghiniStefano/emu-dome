<script setup lang="ts">
// A key or a controller button named in the interface (KbdVariant, KbdSize): a <kbd> in the
// interface's own type. Its label comes from the caller (a key name, a button of the input in use).
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '@/lib/cn'
import type { KbdSize, KbdVariant } from './variants'
import { kbdVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: KbdVariant
  size?: KbdSize
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), { as: 'kbd' })
</script>

<template>
  <Primitive
    data-slot="kbd"
    :as="as"
    :as-child="asChild"
    :class="cn(kbdVariants({ variant, size }), props.class)"
  >
    <slot />
  </Primitive>
</template>
