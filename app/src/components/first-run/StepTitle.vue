<script setup lang="ts">
// A step's title: display type; on the phone 26 / 32 (spec Decision 5: Cowork places it).
defineProps<{ phone: boolean }>()
</script>

<template>
  <h1
    class="m-0"
    :class="phone ? 'text-[26px]/[32px] font-semibold tracking-[-0.02em]' : 'text-display'"
  >
    <slot />
  </h1>
</template>
