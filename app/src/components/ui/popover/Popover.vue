<script setup lang="ts">
// A panel that opens under a button and closes on B, Esc or a click outside, on Reka's Popover
// (dismissable layer, `role="dialog"`, portal). The screen holds `open` and keeps its own button:
// there is no Reka trigger, so A on the controller and a click both toggle it the same way
// (nav.ts clicks; Reka's trigger would act on the pointer, not on a click). PopoverContent hangs
// from that button (`anchor`). How it fits the app's navigation is in PopoverContent.
import type { PopoverRootEmits, PopoverRootProps } from 'reka-ui'
import { PopoverRoot, useForwardPropsEmits } from 'reka-ui'

const props = defineProps<PopoverRootProps>()
const emits = defineEmits<PopoverRootEmits>()
const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <PopoverRoot v-bind="forwarded">
    <slot />
  </PopoverRoot>
</template>
