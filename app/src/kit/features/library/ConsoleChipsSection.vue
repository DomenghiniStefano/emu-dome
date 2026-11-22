<script setup lang="ts">
// ConsoleChips (library/ConsoleChips.vue): the console filter over the grid.
import { ref } from 'vue'
import type { ConsoleId } from '@/emu'
import ConsoleChips from '@/components/library/ConsoleChips.vue'
import type { ConsoleChip } from '@/components/library/libraryView'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { KitGroup } from '@/kit/review'

const all = ref<ConsoleId | 'all'>('all')
const two = ref<ConsoleId | 'all'>('nds')
const EVERY: ConsoleChip[] = [
  { id: 'all', count: 1284 },
  { id: 'gb', count: 211 },
  { id: 'gbc', count: 187 },
  { id: 'gba', count: 612 },
  { id: 'nds', count: 251 },
  { id: '3ds', count: 23 },
]
const FEW: ConsoleChip[] = [
  { id: 'all', count: 4 },
  { id: 'gba', count: 3 },
  { id: 'nds', count: 1 },
]
</script>

<template>
  <KitSection title="Console chips" :group="KitGroup.Library">
    <KitState label="Every console, large counts">
      <ConsoleChips v-model="all" :chips="EVERY" :initial="false" />
    </KitState>
    <KitState label="Two consoles, DS chosen">
      <ConsoleChips v-model="two" :chips="FEW" :initial="false" />
    </KitState>
  </KitSection>
</template>
