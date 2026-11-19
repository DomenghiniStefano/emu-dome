<script setup lang="ts">
// A part of a game's saves (V02): its small uppercase title over its content (the slot).
defineProps<{ title: string }>()
</script>

<template>
  <section class="flex flex-col gap-10">
    <h3 class="m-0 text-overline text-ink-muted">{{ title }}</h3>
    <slot />
  </section>
</template>
