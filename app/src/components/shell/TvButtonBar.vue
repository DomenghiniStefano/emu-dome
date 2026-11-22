<script setup lang="ts">
// The button bar at the bottom in TV mode (#30): what the connected controller's buttons do here,
// each with its glyph (TvGlyph). Words only: the buttons do it, the bar does not take the focus.
import { useI18n } from 'vue-i18n'
import TvGlyph from './TvGlyph.vue'
import type { TvHint } from './tvHints'

defineProps<{ hints: TvHint[] }>()

const { t } = useI18n()
</script>

<template>
  <footer
    class="flex shrink-0 basis-tv-bar flex-wrap items-center justify-end gap-x-tv-hint-gap overflow-hidden px-tv-edge text-tv-hint text-ink-muted"
  >
    <span v-for="h in hints" :key="h.label" class="inline-flex items-center gap-10">
      <span class="inline-flex gap-6">
        <TvGlyph v-for="a in h.actions" :key="a" :action="a" />
      </span>
      {{ t(h.label) }}
    </span>
  </footer>
</template>
