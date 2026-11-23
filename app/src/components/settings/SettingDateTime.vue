<script setup lang="ts">
// A date and time (`dateTime`, `saves.rtcdate`): the DateTimeField primitive, its value in words
// in the UI language. The setting is written when its popover closes, once.
import { useI18n } from 'vue-i18n'
import { DateTimeField, DateTimeFieldVariant } from '@/design-system/components/date-time-field'
import { settingDateTimeLabel } from '@/i18n/labels'

defineProps<{
  label: string
  value: string
  minYear: number
  maxYear: number
  disabled: boolean
}>()
const emit = defineEmits<{ set: [string] }>()
const { locale } = useI18n()
</script>

<template>
  <DateTimeField
    :model-value="value"
    :variant="DateTimeFieldVariant.Field"
    :label="label"
    :text="settingDateTimeLabel(value)"
    :min-year="minYear"
    :max-year="maxYear"
    :locale="locale"
    :disabled="disabled"
    @update:model-value="v => emit('set', v)"
  />
</template>
