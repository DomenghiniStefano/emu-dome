<script setup lang="ts">
import { ref } from 'vue'
import { Slider, SliderVariant } from '@/components/ui/slider'
import { Switch } from '@/components/ui/switch'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'

const volume = ref(70)
const committed = ref(70)
</script>

<template>
  <KitSection title="Slider">
    <KitState label="Mid value, with its text beside (the caller's); commit on release">
      <div class="flex flex-wrap items-center gap-12">
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
    <KitState label="EdgeFill (settings rows): the fill reaches the track's edge">
      <Slider :model-value="60" :variant="SliderVariant.EdgeFill" label="Strength" />
    </KitState>
    <template #review>
      <KitIssue title="In Chiaro il binario quasi sparisce" :status="IssueStatus.Proposed">
        Il binario è surface-raised con il bordo line: in Chiaro contro una card bianca fa 1,26:1, e
        la manopola bianca si vede solo dalla sua ombra. Lo Switch spento accanto, nella stessa riga
        delle impostazioni, ha il bordo line-strong e si legge bene. Passa il Kit a Chiaro e guarda
        «Min»: della corsa non resta quasi niente.
        <template #current>
          <div class="flex items-center gap-12 rounded-card bg-surface p-12">
            <Slider :model-value="0" label="Track, now" />
            <Switch :model-value="false" aria-label="Beside it" />
          </div>
        </template>
        <template #proposed>
          <div class="flex items-center gap-12 rounded-card bg-surface p-12">
            <Slider
              :model-value="0"
              label="Track, proposed"
              class="[&_[data-slot=slider-track]]:border-line-strong"
            />
            <Switch :model-value="false" aria-label="Beside it" />
          </div>
        </template>
        <template #solution>
          Il bordo del binario passa a line-strong in slider/variants.ts (anche per EdgeFill), come
          lo Switch spento.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
