<script setup lang="ts">
// One optional import of S06: what it is, a line or two below, and its action at the end.
defineProps<{ name: string }>()
</script>

<template>
  <div class="flex items-center gap-12 px-16 py-12">
    <div class="min-w-0 flex-1">
      <div class="font-medium">{{ name }}</div>
      <slot />
    </div>
    <slot name="action" />
  </div>
</template>
