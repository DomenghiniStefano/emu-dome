<script setup lang="ts">
// A list choice (`sel` on the Settings board): a field with the current value that opens the
// options in a small glass menu, on the Select primitive (Reka). B / Esc / a click outside close
// it and the focus returns to the field. Also used by the first run (language) and by the scope
// bar of Settings, where it sits in a level's title (`inTitle`): there the list hangs from the
// title's block.
import { computed, ref } from 'vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectTriggerVariant,
  SelectValue,
} from '@/design-system/components/select'

const props = defineProps<{
  label: string
  value: string
  options: { id: string; text: string }[]
  disabled?: boolean
  /** Inside a title: no edge or fill, the title's colour and weight. */
  inTitle?: boolean
}>()
const emit = defineEmits<{ pick: [string] }>()
const root = ref<HTMLElement | null>(null)

/** The current option's text (the stored id when this device does not offer it). */
const current = computed(() => props.options.find(o => o.id === props.value)?.text ?? props.value)
</script>

<template>
  <div ref="root">
    <Select
      :model-value="value"
      :disabled="disabled"
      @update:model-value="v => typeof v === 'string' && emit('pick', v)"
    >
      <SelectTrigger
        :variant="inTitle ? SelectTriggerVariant.InTitle : SelectTriggerVariant.Field"
        :aria-label="label"
      >
        <SelectValue>{{ current }}</SelectValue>
      </SelectTrigger>
      <SelectContent :aria-label="label" :anchor="inTitle ? root : null">
        <SelectItem v-for="o in options" :key="o.id" :value="o.id">{{ o.text }}</SelectItem>
      </SelectContent>
    </Select>
  </div>
</template>
