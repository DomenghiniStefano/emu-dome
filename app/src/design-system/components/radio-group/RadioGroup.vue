<script setup lang="ts">
// A one-of-several choice on Reka's RadioGroup (the settings scope bar's levels, the cover
// picker's covers, the library's order): `radiogroup` / `radio` roles, `aria-checked`, roving
// focus. The arrows along `orientation` move and choose (Reka); no looping, so past the last item
// nav.ts moves on, as on Tabs and Segmented. A click, Space or A chooses. Entering the group from
// outside, nav.ts lands on the chosen item (Reka would choose whatever item a keyboard arrow
// focused).
//
// No look of its own: each item is `as-child` on the screen's own primitive (a Card, a Button),
// which keeps its look and its `selected` state.
import type { RadioGroupRootEmits, RadioGroupRootProps } from 'reka-ui'
import { RadioGroupRoot, useForwardPropsEmits } from 'reka-ui'

const props = withDefaults(defineProps<RadioGroupRootProps>(), {
  loop: false,
  orientation: 'horizontal',
})
const emits = defineEmits<RadioGroupRootEmits>()
const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <RadioGroupRoot v-bind="forwarded">
    <slot />
  </RadioGroupRoot>
</template>
