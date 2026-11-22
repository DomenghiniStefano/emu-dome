<script setup lang="ts">
// A stand-in for the game behind what shows over it (Kit page only): the cover colours as a
// picture, darkened by the paused game's scrim when the quick menu is open, so glass reads as it
// does in the app. With `panel`, the content sits in the quick menu's glass panel (the material
// and edge of quick-menu/panel.ts's Menu layout, without its centring). It holds `fixed`
// children (the HUD, the hint, the toasts) as their containing block.
withDefaults(defineProps<{ panel?: boolean; scrim?: boolean }>(), {
  panel: false,
  scrim: true,
})
</script>

<template>
  <div
    class="relative isolate flex w-full max-w-full overflow-hidden rounded-card bg-letterbox p-20 transform-gpu"
  >
    <div class="absolute inset-0 bg-linear-to-br from-cover-1 via-cover-3 to-cover-2"></div>
    <div class="absolute top-16 right-24 size-48 bg-accent"></div>
    <div class="absolute bottom-24 left-32 h-16 w-panel bg-success"></div>
    <div v-if="scrim" class="absolute inset-0 bg-game-scrim"></div>
    <div
      v-if="panel"
      class="relative mx-auto flex w-panel max-w-full flex-col gap-16 rounded-16 border border-line p-20 shadow-panel glass-thick-flat"
    >
      <slot />
    </div>
    <div v-else class="relative flex max-w-full">
      <slot />
    </div>
  </div>
</template>
