<script setup lang="ts">
// A warning about a save, on the warning's soft fill with its triangle (E04: the in-game save
// set aside; V05: an import replaces the in-game save). The sentence is the slot.
import { TriangleAlert } from '@lucide/vue'
</script>

<template>
  <div class="flex gap-8 rounded-card bg-warning-soft px-10 py-8 text-caption">
    <TriangleAlert
      class="flex-none text-warning"
      :size="16"
      :stroke-width="1.75"
      aria-hidden="true"
    />
    <span><slot /></span>
  </div>
</template>
