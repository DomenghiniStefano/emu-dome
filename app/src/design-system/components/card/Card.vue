<script setup lang="ts">
// A card. Not focusable by itself: a card that is one choice among several is rendered `as`
// a button (`as="button"`), and then nav.ts moves to it and clicks it like any button.
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import type { CardPad, CardVariant } from './variants'
import { cardVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: CardVariant
  /** The room inside, when not the variant's own (CardPad). */
  pad?: CardPad
  /** The chosen card of a choice (Raised): accent edge on accent-soft. */
  selected?: boolean
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), { as: 'div' })
</script>

<template>
  <Primitive
    data-slot="card"
    :as="as"
    :as-child="asChild"
    :type="as === 'button' && !asChild ? 'button' : undefined"
    :data-selected="selected ? '' : undefined"
    :class="cn(cardVariants({ variant, pad }), props.class)"
  >
    <slot />
  </Primitive>
</template>
