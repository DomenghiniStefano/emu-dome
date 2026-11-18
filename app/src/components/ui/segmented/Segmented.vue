<script setup lang="ts">
// A one-of-a-few choice drawn as segments (SettingRow `.segments`), on Reka's ToggleGroup:
// `type="single"`, horizontal, no looping (past the last segment nav.ts moves on). A setting always
// has a value, so a click on the chosen segment, which Reka reports as no value, is ignored.
import type { ToggleGroupRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { ToggleGroupRoot } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  Pick<ToggleGroupRootProps, 'disabled'> & { class?: HTMLAttributes['class'] }
>()
const model = defineModel<string>({ required: true })

function choose(value: unknown) {
  if (typeof value === 'string' && value !== '') model.value = value
}
</script>

<template>
  <ToggleGroupRoot
    data-slot="segmented"
    type="single"
    orientation="horizontal"
    :loop="false"
    :model-value="model"
    :disabled="props.disabled"
    :class="
      cn('flex flex-wrap gap-2 rounded-6 border border-line bg-surface-raised p-2', props.class)
    "
    @update:model-value="choose"
  >
    <slot />
  </ToggleGroupRoot>
</template>
