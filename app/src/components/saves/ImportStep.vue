<script setup lang="ts">
// One step of an import (V05): its number (green once done), what it asks and what is chosen,
// and the step's button (the slot).
defineProps<{ n: number; title: string; detail: string; done: boolean }>()
</script>

<template>
  <div class="flex items-center gap-8">
    <span
      class="grid size-24 flex-none place-items-center rounded-full text-caption font-semibold"
      :class="done ? 'bg-success-soft text-success' : 'bg-surface-raised text-ink-muted'"
    >
      {{ n }}
    </span>
    <div class="min-w-0 flex-1 wrap-anywhere">
      <div class="font-medium">{{ title }}</div>
      <div class="text-caption text-ink-muted">{{ detail }}</div>
    </div>
    <slot />
  </div>
</template>
