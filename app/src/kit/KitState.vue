<script setup lang="ts">
// One state of a primitive on the Kit page, labelled underneath.
defineProps<{ label: string }>()
</script>

<template>
  <figure class="m-0 flex flex-col items-start gap-6">
    <slot />
    <figcaption class="text-caption text-ink-faint">{{ label }}</figcaption>
  </figure>
</template>
