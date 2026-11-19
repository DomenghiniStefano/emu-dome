<script setup lang="ts">
// «Prova i pulsanti»: the grid of 12 buttons of C01 and C02 (boards Controlli2 and Controlli).
// Every cell is a position of the controller (or a key of the keyboard); the held ones light up.
// The text under it (last press, capture hint) belongs to the screen that shows it; what is held
// comes from `useButtonTester` (components/controls).
//
// The cells' 8px radius and their 12px label on the 18px line are pending tokens (`radius-8`,
// `caption-tall`), until Cowork places them (spec Decision 5).
defineProps<{
  cells: { id: string; label: string }[]
  held: Set<string>
  label: string
}>()
</script>

<template>
  <div class="grid w-full max-w-tester grid-cols-4 gap-8" role="img" :aria-label="label">
    <span
      v-for="c in cells"
      :key="c.id"
      :data-lit="held.has(c.id) ? '' : undefined"
      class="grid h-control-lg place-items-center truncate rounded-8 border border-line bg-surface-raised px-4 text-caption-tall font-semibold data-lit:border-accent-text data-lit:bg-accent-soft data-lit:text-accent-text"
    >
      {{ c.label }}
    </span>
  </div>
</template>
