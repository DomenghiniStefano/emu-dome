<script setup lang="ts">
// A block under its small uppercase label (HeadingVariant.Overline): a part of a game's saves,
// a settings section, the quick menu's saves / speed / tools, C01's shortcuts. Beside the label,
// at the far end, an optional `aside` (a link «Altri slot», a note, a hint). One gap between the
// head and the block (`block-gap`). Its words come translated from the caller.
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '@/lib/cn'
import { Heading, HeadingVariant } from '@/components/ui/heading'

interface Props extends PrimitiveProps {
  /** None: the block shows without its head (the quick menu's saves page). */
  title?: string
  /** The label's tag (the outline of the page around it). */
  titleAs?: string
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  as: 'section',
  title: undefined,
  titleAs: 'h2',
  class: undefined,
})
</script>

<template>
  <Primitive
    data-slot="labelled-block"
    :as="as"
    :as-child="asChild"
    :class="cn('flex flex-col gap-block-gap', props.class)"
  >
    <div
      v-if="title || $slots.aside"
      class="flex flex-wrap items-baseline justify-between gap-x-8 gap-y-2"
    >
      <Heading v-if="title" :as="titleAs" :variant="HeadingVariant.Overline">{{ title }}</Heading>
      <slot name="aside" />
    </div>
    <slot />
  </Primitive>
</template>
