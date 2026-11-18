<script setup lang="ts">
// A list choice (SettingSelect) on Reka's Select. The field acts on its own Enter (nav.ts sends A
// as Enter there), the open list owns every arrow, and its content is a nav scope whose `back`
// closes it (SelectContent).
import type { SelectRootEmits, SelectRootProps } from 'reka-ui'
import { SelectRoot, useForwardPropsEmits } from 'reka-ui'

const props = defineProps<SelectRootProps>()
const emits = defineEmits<SelectRootEmits>()
const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <SelectRoot v-bind="forwarded">
    <slot />
  </SelectRoot>
</template>
