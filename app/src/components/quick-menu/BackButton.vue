<script setup lang="ts">
// «‹ Menu rapido»: back one level (B / Esc). On the phone «‹ Indietro» (owner, 2026-11-22: icon
// and name), at the touch minimum.
import { ChevronLeft } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { FocusMark } from './view'
import { Kbd, KbdSize } from '@/design-system/components/kbd'

defineProps<{
  /** Where back leads: the button's text. */
  label: string
  /** The key that goes back with the input in use, or null. */
  hint: string | null
}>()
defineEmits<{ click: [] }>()

const mark = { [FocusMark.Back]: '' }
</script>

<template>
  <Button
    v-bind="mark"
    :variant="ButtonVariant.Ghost"
    :size="ButtonSize.Back"
    class="-ml-8 self-start"
    @click="$emit('click')"
  >
    <ChevronLeft :size="18" :stroke-width="1.75" aria-hidden="true" />
    {{ label }}
    <Kbd v-if="hint" :size="KbdSize.Small" class="ml-4">{{ hint }}</Kbd>
  </Button>
</template>
