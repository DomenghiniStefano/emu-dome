<script setup lang="ts">
// A short text (`text`): written when the field is left or Enter is pressed (`change`).
import { Input, InputVariant } from '@/design-system/components/input'

defineProps<{ label: string; value: string; placeholder: string; disabled: boolean }>()
const emit = defineEmits<{ set: [string] }>()
</script>

<template>
  <Input
    :variant="InputVariant.Field"
    :model-value="value"
    :aria-label="label"
    :placeholder="placeholder"
    :disabled="disabled"
    @change="(e: Event) => emit('set', (e.target as HTMLInputElement).value)"
  />
</template>
