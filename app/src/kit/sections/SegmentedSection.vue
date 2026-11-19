<script setup lang="ts">
import { ref } from 'vue'
import { Segmented, SegmentedItem, SegmentedVariant } from '@/components/ui/segmented'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'

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
    <KitState label="Strong">
      <Segmented v-model="speed" :variant="SegmentedVariant.Strong" aria-label="Speed">
        <SegmentedItem value="normal">Normal</SegmentedItem>
        <SegmentedItem value="fast">Fast forward</SegmentedItem>
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
    <KitState label="Stacked (a name over its value): desktop, and on the phone">
      <div class="flex w-field flex-col gap-8">
        <div
          v-for="density in ['desktop', 'touch']"
          :key="density"
          :data-density="density"
          class="flex flex-col"
        >
          <Segmented v-model="speed" :variant="SegmentedVariant.Stacked" aria-label="Speed">
            <SegmentedItem v-for="id in ['normal', 'fast', 'slow']" :key="id" :value="id">
              <span class="font-semibold">{{ id }}</span>
              <span class="text-caption">1x</span>
            </SegmentedItem>
          </Segmented>
        </div>
      </div>
    </KitState>
  </KitSection>
</template>
