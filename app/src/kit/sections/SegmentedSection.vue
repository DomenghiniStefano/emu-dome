<script setup lang="ts">
import { ref } from 'vue'
import { Segmented, SegmentedItem } from '@/components/ui/segmented'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const speed = ref('normal')
</script>

<template>
  <KitSection title="Segmented">
    <KitState label="One chosen">
      <Segmented v-model="speed" aria-label="Speed">
        <SegmentedItem value="normal">Normal</SegmentedItem>
        <SegmentedItem value="fast">Fast forward</SegmentedItem>
        <SegmentedItem value="slow">Slow motion</SegmentedItem>
      </Segmented>
    </KitState>
    <KitState label="One item disabled">
      <Segmented v-model="speed" aria-label="Speed">
        <SegmentedItem value="normal">Normal</SegmentedItem>
        <SegmentedItem value="fast" disabled>Fast forward</SegmentedItem>
      </Segmented>
    </KitState>
    <KitState label="Disabled">
      <Segmented model-value="normal" disabled aria-label="Speed">
        <SegmentedItem value="normal">Normal</SegmentedItem>
        <SegmentedItem value="fast">Fast forward</SegmentedItem>
      </Segmented>
    </KitState>
  </KitSection>
</template>
