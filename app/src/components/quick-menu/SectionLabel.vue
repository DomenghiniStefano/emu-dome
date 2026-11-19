<script setup lang="ts">
// A section's label (SALVATAGGI RAPIDI, VELOCITÀ, STRUMENTI): the `text-overline` type, one step
// larger on the phone's sheet.
defineProps<{ touch?: boolean }>()
</script>

<template>
  <span
    class="text-ink-muted"
    :class="
      touch
        ? 'text-(length:--text-caption) leading-[14px] font-semibold tracking-(--text-overline--letter-spacing)'
        : 'text-overline'
    "
  >
    <slot />
  </span>
</template>
