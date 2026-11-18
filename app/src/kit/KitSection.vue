<script setup lang="ts">
// One primitive on the Kit page: its name, then its states side by side.
defineProps<{ title: string }>()
</script>

<template>
  <section class="flex flex-col gap-12 border-t border-line pt-16">
    <h2 class="m-0 text-heading text-ink">{{ title }}</h2>
    <div class="flex flex-wrap items-start gap-16">
      <slot />
    </div>
  </section>
</template>
