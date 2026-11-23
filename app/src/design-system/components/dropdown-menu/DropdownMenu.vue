<script setup lang="ts">
// A short menu of actions under a button (a game's «…», L01 and L04), on Reka's DropdownMenu
// (`role="menu"` / `menuitem`, the arrows among the items, typeahead, Esc and a click outside
// close it). Not modal: the page around it keeps scrolling and taking clicks, as before.
// The screen holds `open` and keeps its own button: there is no Reka trigger, so A on the
// controller and a click both toggle it the same way (nav.ts clicks; Reka's trigger acts on the
// pointer). DropdownMenuContent hangs from that button (`anchor`).
import type { DropdownMenuRootEmits, DropdownMenuRootProps } from 'reka-ui'
import { DropdownMenuRoot, useForwardPropsEmits } from 'reka-ui'

const props = withDefaults(defineProps<DropdownMenuRootProps>(), { modal: false })
const emits = defineEmits<DropdownMenuRootEmits>()
const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <DropdownMenuRoot v-bind="forwarded">
    <slot />
  </DropdownMenuRoot>
</template>
