<script setup lang="ts">
// An on / off setting (`toggle`).
import { Switch } from '@/components/ui/switch'

defineProps<{ label: string; on: boolean; disabled: boolean }>()
const emit = defineEmits<{ set: [boolean] }>()
</script>

<template>
  <Switch
    :model-value="on"
    :aria-label="label"
    :disabled="disabled"
    @update:model-value="(v: boolean) => emit('set', v)"
  />
</template>
