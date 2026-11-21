<script setup lang="ts">
// The settings a page changes «in prova» (Session level), as compact setting rows.
import type { SettingEntry, SettingKey, SettingRow, SettingsSchema, SettingValue } from '@/emu'
import SettingRowView from '@/components/settings/SettingRow.vue'

defineProps<{
  entries: SettingEntry[]
  rows: Record<string, SettingRow>
  schema: SettingsSchema
}>()
const emit = defineEmits<{ set: [SettingKey, SettingValue]; error: [string] }>()
</script>

<template>
  <SettingRowView
    v-for="e in entries"
    :key="e.key"
    :entry="e"
    :row="rows[e.key]"
    :schema="schema"
    level="session"
    :level-name="() => ''"
    :scoped="false"
    compact
    @set="v => emit('set', e.key, v)"
    @error="text => emit('error', text)"
  />
</template>
