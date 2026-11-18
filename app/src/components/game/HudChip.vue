<script setup lang="ts">
// One discreet indicator over the game (speed, pause, FPS): thick glass, never interactive.
</script>

<template>
  <div
    class="flex h-control-sm items-center rounded-6 border border-line bg-glass-thick px-10 font-semibold"
  >
    <slot />
  </div>
</template>
