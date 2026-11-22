<script setup lang="ts">
// One banner over the library's grid (the scan running, what it added, a folder not reachable):
// the same edge, corners and room for all of them, neutral or warning. It wraps in a narrow
// window: what does not fit goes under, never squeezed into a column of letters.
defineProps<{ warning?: boolean }>()
</script>

<template>
  <div
    class="flex flex-wrap items-center gap-x-12 gap-y-6 rounded-card border px-10 py-6"
    :class="warning ? 'border-warning-edge bg-warning-tint' : 'border-line bg-surface'"
    role="status"
  >
    <slot />
  </div>
</template>
