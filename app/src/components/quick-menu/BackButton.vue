<script setup lang="ts">
// «‹ Menu rapido»: back one level (B / Esc). On the phone's sheet the chevron alone, at the
// touch minimum.
import { ChevronLeft } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import KeyHint from './KeyHint.vue'
import { FocusMark } from './view'

defineProps<{
  /** Where back leads: the button's text, or its accessible name when the chevron is alone. */
  label: string
  /** The key that goes back with the input in use, or null. */
  hint: string | null
  iconOnly?: boolean
}>()
defineEmits<{ click: [] }>()

const mark = { [FocusMark.Back]: '' }
</script>

<template>
  <Button
    v-if="iconOnly"
    v-bind="mark"
    :variant="ButtonVariant.Ghost"
    :size="ButtonSize.IconTouch"
    class="-ml-10 self-start"
    :aria-label="label"
    :title="label"
    @click="$emit('click')"
  >
    <ChevronLeft :size="22" :stroke-width="1.75" aria-hidden="true" />
  </Button>
  <Button
    v-else
    v-bind="mark"
    :variant="ButtonVariant.Ghost"
    :size="ButtonSize.Back"
    class="self-start"
    @click="$emit('click')"
  >
    <ChevronLeft :size="18" :stroke-width="1.75" aria-hidden="true" />
    {{ label }}
    <KeyHint v-if="hint" :label="hint" />
  </Button>
</template>
