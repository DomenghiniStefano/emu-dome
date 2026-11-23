<script setup lang="ts">
// One component on the Kit page: its name, its states side by side, then its review issues
// (KitIssue in the #review slot). Registers on the page's board (review.ts) for the index and the
// «only with issues» filter.
import { KitGroup, useKitSection } from '@/design-system/kit/review'

const props = withDefaults(defineProps<{ title: string; group?: KitGroup }>(), {
  group: KitGroup.Primitives,
})
const { entry, hidden } = useKitSection(
  () => props.title,
  () => props.group,
)
</script>

<template>
  <section v-show="!hidden" :id="entry.id" class="flex flex-col gap-8 border-t border-line pt-12">
    <h2 class="m-0 text-heading text-ink">{{ title }}</h2>
    <div class="flex flex-wrap items-start gap-12">
      <slot />
    </div>
    <div v-if="$slots.review" class="flex flex-col gap-8">
      <slot name="review" />
    </div>
  </section>
</template>
