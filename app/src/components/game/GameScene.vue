<script setup lang="ts">
// The full-window scene while a game starts (G01) or closes (G14): its title over the cover's
// colour, a line below, and a bar that keeps moving while the work has no known length.
// Layer 35, between the overlays and the dialogs (spec Decision 5: Cowork names it).
defineProps<{
  title: string
  /** The cover's colour; none: the surface's. */
  colour?: string | null
}>()
</script>

<template>
  <div
    class="fixed inset-0 z-[35] flex flex-col items-center justify-center gap-10 bg-cover-glow"
    :style="{ '--cover': colour ?? 'var(--surface)' }"
    role="status"
  >
    <div class="px-16 text-center text-display">{{ title }}</div>
    <slot />
    <div class="h-4 w-[200px] overflow-hidden rounded-full bg-surface-raised" aria-hidden="true">
      <span class="block h-full w-2/5 animate-progress-slide rounded-full bg-accent-text"></span>
    </div>
  </div>
</template>
