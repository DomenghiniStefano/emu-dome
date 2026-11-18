<script setup lang="ts">
import { ref } from 'vue'
import { Slider } from '@/components/ui/slider'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const volume = ref(70)
const committed = ref(70)
</script>

<template>
  <KitSection title="Slider">
    <KitState label="Mid value, with its text beside (the caller's); commit on release">
      <div class="flex items-center gap-12">
        <Slider
          v-model="volume"
          label="Volume"
          :value-text="`${volume}%`"
          @commit="committed = $event"
        />
        <span class="min-w-value text-right font-semibold tabular-nums">{{ volume }}%</span>
        <span class="text-caption text-ink-faint">committed {{ committed }}</span>
      </div>
    </KitState>
    <KitState label="Min">
      <Slider :model-value="0" label="Min" />
    </KitState>
    <KitState label="Max">
      <Slider :model-value="100" label="Max" />
    </KitState>
    <KitState label="Disabled">
      <Slider :model-value="40" disabled label="Disabled" />
    </KitState>
  </KitSection>
</template>
