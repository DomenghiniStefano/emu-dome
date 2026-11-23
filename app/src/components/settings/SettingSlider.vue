<script setup lang="ts">
// A number on a range (`slider`), with its value in words beside it. The setting is written when
// a change ends (a release, a key), not on every step of a drag.
import { ref, watch } from 'vue'
import { Slider, SliderVariant } from '@/design-system/components/slider'

const props = defineProps<{
  label: string
  value: number
  min: number
  max: number
  step: number
  /** The value with its unit («70%»). */
  valueText: string
  disabled: boolean
}>()
const emit = defineEmits<{ set: [number] }>()

const shown = ref(props.value)
watch(
  () => props.value,
  v => (shown.value = v),
)
</script>

<template>
  <!-- On a touch screen the slider takes its whole line (SettingRow gives it the row's width), its
       value at the end: no gap between them (#127). -->
  <div class="flex items-center gap-8 touch:w-full">
    <Slider
      v-model="shown"
      class="touch:w-auto touch:flex-1"
      :variant="SliderVariant.EdgeFill"
      :min="min"
      :max="max"
      :step="step"
      :label="label"
      :value-text="valueText"
      :disabled="disabled"
      @commit="(v: number) => emit('set', v)"
    />
    <span class="min-w-value text-right font-semibold tabular-nums">{{ valueText }}</span>
  </div>
</template>
