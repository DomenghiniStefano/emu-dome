<script setup lang="ts">
import { ref } from 'vue'
import { Chip, ChipVariant } from '@/components/ui/chip'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'

const filter = ref('all')
const options = ref(['png'])
const toggle = (id: string) =>
  (options.value = options.value.includes(id)
    ? options.value.filter(o => o !== id)
    : [...options.value, id])
</script>

<template>
  <KitSection title="Chip">
    <KitState label="Filter · selected / not (aria-pressed)">
      <div class="flex gap-8">
        <Chip
          v-for="id in ['all', 'gba', 'nds']"
          :key="id"
          :selected="filter === id"
          :aria-pressed="filter === id"
          @click="filter = id"
        >
          {{ id }}
        </Chip>
      </div>
    </KitState>
    <KitState label="Option · selected / not (checkbox)">
      <div class="flex gap-8">
        <Chip
          v-for="id in ['png', 'bmp']"
          :key="id"
          :variant="ChipVariant.Option"
          :selected="options.includes(id)"
          role="checkbox"
          :aria-checked="options.includes(id)"
          @click="toggle(id)"
        >
          {{ id }}
        </Chip>
      </div>
    </KitState>
    <KitState label="Option · shown only (as span)">
      <Chip as="span" :variant="ChipVariant.Option">1 png</Chip>
    </KitState>
    <KitState label="Filter · disabled">
      <Chip disabled>3ds</Chip>
    </KitState>
  </KitSection>
</template>
