<script setup lang="ts">
// «Prova i pulsanti»: the grid of 12 buttons of C01 and C02 (boards Controlli2 and Controlli).
// Every cell is a position of the controller (or a key of the keyboard); the held ones light up.
// The text under it (last press, capture hint) belongs to the screen that shows it.
defineProps<{
  cells: { id: string; label: string }[]
  held: Set<string>
  label: string
}>()
</script>

<template>
  <div class="cells" role="img" :aria-label="label">
    <span v-for="c in cells" :key="c.id" class="cell" :class="{ lit: held.has(c.id) }">
      {{ c.label }}
    </span>
  </div>
</template>

<style scoped>
.cells {
  width: 100%;
  max-width: 280px;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}
.cell {
  height: 40px;
  border-radius: 8px;
  border: 1px solid var(--line);
  background: var(--surface-raised);
  font-weight: 600;
  font-size: 12px;
  display: grid;
  place-items: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 0 4px;
}
.cell.lit {
  color: var(--accent-text);
  border-color: var(--accent-text);
  background: var(--accent-soft);
}
</style>
