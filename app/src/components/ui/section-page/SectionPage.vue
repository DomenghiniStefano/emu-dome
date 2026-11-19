<script setup lang="ts">
// The page of a section beside the rail (Settings, Controls, Saves, a game's details): its room
// from the edges, the gap between its blocks, how wide they grow (SectionWidth), and whether it
// scrolls itself. One place for all of it: a section screen never writes its own padding.
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '@/lib/cn'
import type { SectionWidth } from './variants'
import { sectionPageVariants } from './variants'

interface Props extends PrimitiveProps {
  width?: SectionWidth
  scroll?: boolean
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), { as: 'div', scroll: true })
</script>

<template>
  <Primitive
    data-slot="section-page"
    :as="as"
    :as-child="asChild"
    :class="cn(sectionPageVariants({ width, scroll }), props.class)"
  >
    <slot />
  </Primitive>
</template>
