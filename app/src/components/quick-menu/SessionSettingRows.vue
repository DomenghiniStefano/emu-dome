<script setup lang="ts">
// The settings a page changes «in prova» (Session level), as compact setting rows; with `level`
// `global`, the ones it sets for this device at once (useDeviceSettings).
import type {
  SettingEntry,
  SettingKey,
  SettingLevel,
  SettingRow,
  SettingsSchema,
  SettingValue,
} from '@/emu'
import SettingRowView from '@/components/settings/SettingRow.vue'

defineProps<{
  entries: SettingEntry[]
  rows: Record<string, SettingRow>
  schema: SettingsSchema
  /** Session (default) or Global (the device rows). */
  level?: SettingLevel
}>()
const emit = defineEmits<{ set: [SettingKey, SettingValue] }>()
</script>

<template>
  <SettingRowView
    v-for="e in entries"
    :key="e.key"
    :entry="e"
    :row="rows[e.key]"
    :schema="schema"
    :level="level ?? 'session'"
    :level-name="() => ''"
    :scoped="false"
    compact
    @set="v => emit('set', e.key, v)"
  />
</template>
