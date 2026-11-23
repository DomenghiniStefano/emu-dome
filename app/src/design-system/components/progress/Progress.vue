<script setup lang="ts">
// A progress bar on Reka's Progress (role="progressbar", aria-valuenow / max). `value` null or
// left out: the total is not known and a band sweeps the track; a number: that share is done.
// Not focusable. `label` names it for assistive technology (already translated by the caller).
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { ProgressIndicator, ProgressRoot } from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import { fillWidth } from './fillWidth'
import { progressIndicatorVariants, progressVariants } from './variants'

const props = withDefaults(
  defineProps<{
    value?: number | null
    max?: number
    label: string
    class?: HTMLAttributes['class']
  }>(),
  { value: null, max: 100, class: undefined },
)

const width = computed(() => fillWidth(props.value, props.max))
</script>

<template>
  <ProgressRoot
    data-slot="progress"
    :model-value="value"
    :max="max"
    :get-value-label="() => label"
    :class="cn(progressVariants(), props.class)"
  >
    <ProgressIndicator
      :class="progressIndicatorVariants({ indeterminate: width === null })"
      :style="width === null ? undefined : { width }"
    />
  </ProgressRoot>
</template>
