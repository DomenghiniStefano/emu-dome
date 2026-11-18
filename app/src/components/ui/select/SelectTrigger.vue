<script setup lang="ts">
// The field showing the chosen option (SettingSelect `.field`): 200px at least, 32px high, a
// chevron at the end.
import type { SelectTriggerProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { ChevronDown } from '@lucide/vue'
import { SelectIcon, SelectTrigger } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<SelectTriggerProps & { class?: HTMLAttributes['class'] }>()

const delegated = computed(() => {
  const { class: _class, ...rest } = props
  return rest
})
</script>

<template>
  <SelectTrigger
    data-slot="select-trigger"
    v-bind="delegated"
    :class="
      cn(
        'flex h-control min-w-field cursor-pointer items-center gap-8 rounded-6 border border-line bg-surface-raised px-10 text-left font-sans text-label text-ink enabled:hover:bg-surface-hover disabled:cursor-not-allowed',
        props.class,
      )
    "
  >
    <span class="flex-1"><slot /></span>
    <SelectIcon as-child>
      <ChevronDown :size="14" :stroke-width="1.75" aria-hidden="true" />
    </SelectIcon>
  </SelectTrigger>
</template>
