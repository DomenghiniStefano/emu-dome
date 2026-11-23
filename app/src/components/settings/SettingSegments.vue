<script setup lang="ts">
// A one-of-a-few choice shown as segments (`choice`, style `segments`).
import { Segmented, SegmentedItem } from '@/design-system/components/segmented'
import type { RowOption } from './useSettingRow'

defineProps<{ label: string; value: string; options: RowOption[]; disabled: boolean }>()
const emit = defineEmits<{ set: [string] }>()
</script>

<template>
  <Segmented
    :model-value="value"
    :aria-label="label"
    :disabled="disabled"
    @update:model-value="(v: string) => emit('set', v)"
  >
    <SegmentedItem v-for="o in options" :key="o.id" :value="o.id">{{ o.text }}</SegmentedItem>
  </Segmented>
</template>
