<script setup lang="ts">
// A heading of a settings page and its rows on one card. With `note`, a line beside the heading.
defineProps<{ title: string; note?: string }>()
</script>

<template>
  <section class="flex flex-col gap-8">
    <div v-if="note" class="flex flex-wrap items-baseline justify-between gap-8">
      <h3 class="m-0 px-4 text-overline text-ink-muted">{{ title }}</h3>
      <span class="text-caption text-ink-muted">{{ note }}</span>
    </div>
    <h3 v-else class="m-0 px-4 text-overline text-ink-muted">{{ title }}</h3>
    <div class="flex flex-col rounded-10 border border-line bg-surface">
      <slot />
    </div>
  </section>
</template>
