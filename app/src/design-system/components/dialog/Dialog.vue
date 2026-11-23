<script setup lang="ts">
// A blocking dialog, on Reka's Dialog (focus trap, portal, aria, focus returned to the trigger on
// close). How it is adapted to the app's navigation is in DialogContent.
import type { DialogRootEmits, DialogRootProps } from 'reka-ui'
import { DialogRoot, useForwardPropsEmits } from 'reka-ui'

const props = defineProps<DialogRootProps>()
const emits = defineEmits<DialogRootEmits>()
const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <DialogRoot v-bind="forwarded">
    <slot />
  </DialogRoot>
</template>
