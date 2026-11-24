<script setup lang="ts">
// Closing with a game open (Trello #86), placed over the app: bottom centre, the game visible
// behind (no dimming), on a transparent layer that takes the clicks so nothing else can be used
// meanwhile. The app closing: ClosingStrip (saving, saved). Any close (the app's, or a game's to
// the library under G14) that does not end well: ClosingAsk, the strip grown into a card. State
// in useClosing.ts; the looks live in ClosingStrip.vue and ClosingAsk.vue.
import ClosingAsk from './ClosingAsk.vue'
import ClosingStrip from './ClosingStrip.vue'
import type { ClosingView } from './closing'

defineProps<{ view: ClosingView }>()
const emit = defineEmits<{ wait: []; closeAnyway: [] }>()
</script>

<template>
  <!-- Inside the system's bars (#161): Android shows them again once the game ends, the
       navigation bar at the bottom upright, on a side on its side; 0 where there are none. -->
  <div
    v-if="view.status || view.ask"
    class="fixed inset-0 z-dialog flex items-end justify-center pt-safe-top pr-[calc(var(--spacing-16)+var(--spacing-safe-right))] pb-[calc(var(--spacing-16)+var(--spacing-safe-bottom))] pl-[calc(var(--spacing-16)+var(--spacing-safe-left))]"
  >
    <!-- Keyed by the question: a new one opens afresh and its main action takes the focus. -->
    <ClosingAsk
      v-if="view.ask"
      :key="view.ask.kind"
      :ask="view.ask"
      @wait="emit('wait')"
      @close-anyway="emit('closeAnyway')"
    />
    <ClosingStrip v-else :saved="view.status === 'saved'" />
  </div>
</template>
