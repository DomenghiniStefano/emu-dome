<script setup lang="ts">
// A heading: a page's title, a block's label, a scene's title (HeadingVariant). No strings of its
// own: the words come from the caller, already translated. Not focusable.
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import type { HeadingVariant } from './variants'
import { headingVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: HeadingVariant
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), { as: 'h2' })
</script>

<template>
  <Primitive
    data-slot="heading"
    :as="as"
    :as-child="asChild"
    :class="cn(headingVariants({ variant }), props.class)"
  >
    <slot />
  </Primitive>
</template>
