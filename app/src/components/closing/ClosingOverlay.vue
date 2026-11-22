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
  <div
    v-if="view.status || view.ask"
    class="fixed inset-0 z-dialog flex items-end justify-center px-16 pb-16"
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
