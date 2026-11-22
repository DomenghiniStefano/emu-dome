<script setup lang="ts">
// The full-window scene while a game starts (G01) or closes (G14): its title over the cover's
// colour, a line below, and a bar that keeps moving while the work has no known length (or shows
// how far it is, `value` 0–100: the saves coming down from Drive, G03).
// Layer `scene` (35), between the overlays and the dialogs.
import { Heading, HeadingVariant } from '@/components/ui/heading'
import { Progress } from '@/components/ui/progress'

withDefaults(
  defineProps<{
    title: string
    /** The cover's colour; none: the surface's. */
    colour?: string | null
    value?: number | null
  }>(),
  { colour: null, value: null },
)
</script>

<template>
  <div
    class="fixed inset-0 z-scene flex flex-col items-center justify-center gap-10 bg-cover-glow"
    :style="{ '--cover-tint': colour ?? 'var(--surface)' }"
    role="status"
  >
    <Heading as="div" :variant="HeadingVariant.Display" class="px-16 text-center">{{
      title
    }}</Heading>
    <slot />
    <Progress class="w-progress" :value="value" :label="title" />
  </div>
</template>
