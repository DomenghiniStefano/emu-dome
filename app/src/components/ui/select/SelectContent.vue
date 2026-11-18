<script setup lang="ts">
// The open list (SettingSelectMenu): thick glass under the field, at least as wide as the field,
// 320px at most before it scrolls.
import type { SelectContentEmits, SelectContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { SelectContent, SelectPortal, SelectViewport, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'
import SelectNavScope from './SelectNavScope.vue'

const props = withDefaults(
  defineProps<SelectContentProps & { class?: HTMLAttributes['class'] }>(),
  { position: 'popper', side: 'bottom', align: 'end', sideOffset: 4 },
)
const emits = defineEmits<SelectContentEmits>()

const delegated = computed(() => {
  const { class: _class, ...rest } = props
  return rest
})
const forwarded = useForwardPropsEmits(delegated, emits)
</script>

<template>
  <SelectPortal>
    <SelectContent
      data-slot="select-content"
      v-bind="forwarded"
      :class="
        cn(
          'glass-thick z-menu max-h-menu min-w-(--reka-select-trigger-width) overflow-hidden',
          props.class,
        )
      "
    >
      <SelectNavScope>
        <SelectViewport class="flex flex-col gap-2 p-4">
          <slot />
        </SelectViewport>
      </SelectNavScope>
    </SelectContent>
  </SelectPortal>
</template>
