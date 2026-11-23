<script setup lang="ts">
// A slider (SettingRow `.slider`) on Reka's Slider. One value: `v-model` is a number (Reka's is a
// list). The thumb takes left / right itself, from the keyboard and, through nav.ts's rule for
// Reka, from the controller; up / down leave it. `commit` fires when a change ends (a release,
// a key), which is when SettingRow writes a setting today.
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { SliderRange, SliderRoot, SliderThumb, SliderTrack } from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import { fillPercent, SliderVariant, sliderRangeClass, sliderTrackVariants } from './variants'

const props = withDefaults(
  defineProps<{
    min?: number
    max?: number
    step?: number
    disabled?: boolean
    /** The thumb's accessible name (the setting's name). */
    label?: string
    /** The value as words, for assistive technology («70%»). */
    valueText?: string
    variant?: SliderVariant
    class?: HTMLAttributes['class']
  }>(),
  { min: 0, max: 100, step: 1 },
)
const model = defineModel<number>({ required: true })
const emit = defineEmits<{ commit: [number] }>()

const values = computed({
  get: () => [model.value],
  set: (v: number[] | undefined) => {
    if (v?.[0] !== undefined) model.value = v[0]
  },
})
const edgeFill = computed(() => props.variant === SliderVariant.EdgeFill)
const fill = computed(() => `${fillPercent(model.value, props.min, props.max)}%`)
</script>

<template>
  <!-- 200px wide, a 4px track edged in `line` and filled in accent-text (variants.ts), a 16px
       knob. -->
  <SliderRoot
    v-model="values"
    data-slot="slider"
    :min="min"
    :max="max"
    :step="step"
    :disabled="disabled"
    :class="
      cn(
        'relative flex h-20 w-field touch-none items-center select-none data-[disabled]:cursor-not-allowed',
        props.class,
      )
    "
    @value-commit="(v: number[]) => v[0] !== undefined && emit('commit', v[0])"
  >
    <SliderTrack
      data-slot="slider-track"
      :class="sliderTrackVariants({ variant })"
      :style="edgeFill ? { '--fill': fill } : undefined"
    >
      <SliderRange v-if="!edgeFill" data-slot="slider-range" :class="sliderRangeClass" />
    </SliderTrack>
    <SliderThumb
      data-slot="slider-thumb"
      :aria-label="label"
      :aria-valuetext="valueText"
      class="block size-16 cursor-pointer rounded-full bg-knob shadow-thumb data-[disabled]:cursor-not-allowed"
    />
  </SliderRoot>
</template>
