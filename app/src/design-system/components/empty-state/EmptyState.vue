<script setup lang="ts">
// A place with nothing to show yet (EmptyStateVariant): what is missing and what to do about it.
// No strings of its own: the title and text come translated from the caller. The actions go in
// the default slot (the caller marks the first one `data-nav-initial`), a closing line (formats,
// an error) in `footer`.
import type { Component, HTMLAttributes } from 'vue'
import type { PrimitiveProps } from 'reka-ui'
import { Primitive } from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import { Heading } from '@/design-system/components/heading'
import type { EmptyStateVariant } from './variants'
import { emptyStateVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: EmptyStateVariant
  /** A Lucide icon, shown in a tile above the title. */
  icon?: Component
  title?: string
  /** The title's tag (the outline of the page around it). */
  titleAs?: string
  text?: string
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), { as: 'section', titleAs: 'h2' })
</script>

<template>
  <Primitive
    data-slot="empty-state"
    :as="as"
    :as-child="asChild"
    :class="cn(emptyStateVariants({ variant }), props.class)"
  >
    <span
      v-if="icon"
      class="grid size-48 flex-none place-items-center rounded-card border border-line bg-surface-raised text-ink-muted"
      aria-hidden="true"
    >
      <component :is="icon" :size="24" :stroke-width="1.75" />
    </span>
    <Heading v-if="title" :as="titleAs">{{ title }}</Heading>
    <p v-if="text" class="m-0 max-w-empty text-ink-muted">{{ text }}</p>
    <div v-if="$slots.default" class="mt-4 flex flex-wrap justify-center gap-6">
      <slot />
    </div>
    <slot name="footer" />
  </Primitive>
</template>
