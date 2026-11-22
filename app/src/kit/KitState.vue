<script setup lang="ts">
// One state of a primitive on the Kit page, labelled underneath. Never wider than the page: a
// demo drawn at a fixed width (w-page, w-dialog) takes `max-w-full` and shrinks with it.
defineProps<{ label: string }>()
</script>

<template>
  <figure class="m-0 flex max-w-full min-w-0 flex-col items-start gap-6">
    <slot />
    <figcaption class="text-caption text-ink-faint">{{ label }}</figcaption>
  </figure>
</template>
