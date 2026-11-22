<script setup lang="ts">
// One discreet indicator over the game (speed, pause, FPS): thick glass, never interactive.
</script>

<template>
  <div
    class="flex h-control-sm items-center rounded-card border border-line bg-glass-thick px-8 font-semibold"
  >
    <slot />
  </div>
</template>
