<script setup lang="ts">
// One option (SettingSelectMenu `.option`): a control high, in the list's text colour, the highlighted one
// on surface-hover, the chosen one in accent text on accent-soft.
import type { SelectItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { SelectItem, SelectItemText } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<SelectItemProps & { class?: HTMLAttributes['class'] }>()

const delegated = computed(() => {
  const { class: _class, ...rest } = props
  return rest
})
</script>

<template>
  <SelectItem
    data-slot="select-item"
    v-bind="delegated"
    :class="
      cn(
        'flex h-control shrink-0 cursor-pointer items-center rounded-control px-8 font-sans text-body whitespace-nowrap text-inherit select-none data-[disabled]:cursor-not-allowed data-[disabled]:opacity-locked data-[highlighted]:bg-surface-hover data-[state=checked]:bg-accent-soft data-[state=checked]:font-semibold data-[state=checked]:text-accent-text',
        props.class,
      )
    "
  >
    <SelectItemText>
      <slot />
    </SelectItemText>
  </SelectItem>
</template>
