<script setup lang="ts">
// The key that does an action, beside it (Esc, B): the hints show only the input in use's.
// `onAccent`: on the main action's accent fill.
defineProps<{ label: string; onAccent?: boolean }>()
</script>

<template>
  <span
    class="ml-4 inline-flex h-18 items-center rounded-badge border px-5 text-badge font-semibold"
    :class="onAccent ? 'border-on-accent-edge text-on-accent' : 'border-line-strong text-ink'"
  >
    {{ label }}
  </span>
</template>
