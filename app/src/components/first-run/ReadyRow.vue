<script setup lang="ts">
// One optional import of S06: what it is, a line or two below, and its action at the end. On a
// touch screen the action goes under the words, so neither the words nor the button are squeezed
// on a phone held upright (#127).
defineProps<{ name: string }>()
</script>

<template>
  <div class="flex items-center gap-10 px-12 py-8 touch:flex-col touch:items-start touch:gap-8">
    <div class="min-w-0 flex-1 touch:self-stretch">
      <div class="font-medium">{{ name }}</div>
      <slot />
    </div>
    <slot name="action" />
  </div>
</template>
