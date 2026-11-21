<script setup lang="ts">
// G16 «Imposta le velocità»: fast forward, slow motion and the sound in fast forward, at the
// levels each one accepts.
import { useI18n } from 'vue-i18n'
import type { SettingEntry, SettingKey, SettingRow, SettingsSchema, SettingValue } from '@/emu'
import MenuList from './MenuList.vue'
import SessionSettingRows from './SessionSettingRows.vue'
import { FocusMark } from './view'

defineProps<{
  entries: SettingEntry[]
  rows: Record<string, SettingRow>
  schema: SettingsSchema | null
}>()
const emit = defineEmits<{ set: [SettingKey, SettingValue]; error: [string] }>()

const { t } = useI18n()
const initial = { [FocusMark.SubInitial]: '' }
</script>

<template>
  <MenuList v-bind="initial">
    <SessionSettingRows
      v-if="schema"
      :entries="entries"
      :rows="rows"
      :schema="schema"
      @set="(key, v) => emit('set', key, v)"
      @error="text => emit('error', text)"
    />
  </MenuList>
  <p class="m-0 text-caption text-ink-muted">{{ t('qm.speedsAlsoIn') }}</p>
</template>
