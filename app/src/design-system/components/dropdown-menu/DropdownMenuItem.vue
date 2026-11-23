<script setup lang="ts">
// One action of a DropdownMenu: Reka's menu item behaviour (`role="menuitem"`, highlighted on
// hover, Enter / Space / a click choose it and close the menu) put on its only child, the menu's
// Button (`ButtonVariant.Menu`), which keeps its look.
import type { DropdownMenuItemEmits, DropdownMenuItemProps } from 'reka-ui'
import { DropdownMenuItem, useForwardPropsEmits } from 'reka-ui'

const props = withDefaults(defineProps<DropdownMenuItemProps>(), { asChild: true })
const emits = defineEmits<DropdownMenuItemEmits>()
const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <DropdownMenuItem v-bind="forwarded">
    <slot />
  </DropdownMenuItem>
</template>
