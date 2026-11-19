<script setup lang="ts">
// A Controls page's title (E05, C01, a device's name). The design system's `title` step without
// its -0.01em tracking, which the section has never had (Cowork: adopt it or not).
withDefaults(defineProps<{ as?: 'h1' | 'h2' }>(), { as: 'h1' })
</script>

<template>
  <component :is="as" class="m-0 text-title tracking-[normal]"><slot /></component>
</template>
