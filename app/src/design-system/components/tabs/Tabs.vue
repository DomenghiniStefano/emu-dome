<script setup lang="ts">
// Tabs on Reka's TabsRoot. Activation is manual: moving the focus along the tabs (the arrows, or
// the controller through nav.ts's rule for Reka's roving focus) does not open one; Enter, A or a
// click does. L1 / R1 stay with the screen's `useNavScope(root, { tab })`.
import type { TabsRootEmits, TabsRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed, provide, toRef } from 'vue'
import { TabsRoot, useForwardPropsEmits } from 'reka-ui'
import { TabsSizeKey } from './context'
import { TabsSize } from './variants'

const props = withDefaults(
  defineProps<TabsRootProps & { size?: TabsSize; class?: HTMLAttributes['class'] }>(),
  { size: TabsSize.Default, activationMode: 'manual' },
)
const emits = defineEmits<TabsRootEmits>()

const delegated = computed(() => {
  const { class: _class, size: _size, ...rest } = props
  return rest
})
const forwarded = useForwardPropsEmits(delegated, emits)
provide(TabsSizeKey, toRef(props, 'size'))
</script>

<template>
  <TabsRoot data-slot="tabs" v-bind="forwarded" :class="props.class">
    <slot />
  </TabsRoot>
</template>
