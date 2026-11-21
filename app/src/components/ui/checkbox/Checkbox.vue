<script setup lang="ts">
// An on / off choice in a list of them (L03 «Mostra»), on Reka's Checkbox: `role="checkbox"`,
// `aria-checked`, `data-state`; a click, Space or A toggle it (Enter is left to nav.ts, whose
// `confirm` clicks it). No look of its own: it is `as-child` on the screen's own primitive (a
// Button), which keeps its look. A setting's on / off is a Switch.
import type { CheckboxRootEmits, CheckboxRootProps } from 'reka-ui'
import { CheckboxRoot, useForwardPropsEmits } from 'reka-ui'

const props = withDefaults(defineProps<CheckboxRootProps>(), { asChild: true })
const emits = defineEmits<CheckboxRootEmits>()
const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <CheckboxRoot v-bind="forwarded">
    <slot />
  </CheckboxRoot>
</template>
