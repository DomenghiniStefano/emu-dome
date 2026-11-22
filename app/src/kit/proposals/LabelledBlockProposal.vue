<script setup lang="ts">
// Kit proposal (LabelledBlock): the head always one height (18, the line of a LabelLink and of a
// caption-tall note), and the note drawn by the block from a `note` prop, so every block's
// content starts at the same distance under its label whatever sits beside it. Shown on the Kit
// page only; if the owner takes it, LabelledBlock.vue itself changes.
import { Heading, HeadingVariant } from '@/components/ui/heading'

defineProps<{ title: string; note?: string }>()
</script>

<template>
  <section class="flex flex-col gap-block-gap">
    <div class="flex min-h-18 flex-wrap items-baseline justify-between gap-8">
      <Heading as="h3" :variant="HeadingVariant.Overline">{{ title }}</Heading>
      <span v-if="note" class="text-caption-tall text-ink-muted">{{ note }}</span>
      <slot name="aside" />
    </div>
    <slot />
  </section>
</template>
