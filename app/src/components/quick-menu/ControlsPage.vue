<script setup lang="ts">
// G10 «Controlli»: the controller in use, the touch controls' settings «in prova», on a touch
// screen the row to the C06 touch controls editor (#29), remapping (not yet) and the controller
// shortcuts.
import { useI18n } from 'vue-i18n'
import { ChevronRight } from '@lucide/vue'
import type { Pos, SettingEntry, SettingKey, SettingRow, SettingsSchema, SettingValue } from '@/emu'
import { Card, CardVariant } from '@/components/ui/card'
import { posName } from '@/lib/controls'
import MenuList from './MenuList.vue'
import SessionSettingRows from './SessionSettingRows.vue'
import { FocusMark } from './view'

defineProps<{
  /** The controller in use, null with none. */
  controller: string | null
  entries: SettingEntry[]
  rows: Record<string, SettingRow>
  schema: SettingsSchema | null
  modifier: Pos
  /** The shortcuts that have a button. */
  shortcutCount: number
  /** A touch screen: the row that opens the C06 touch controls editor (#29). */
  touch: boolean
}>()
const emit = defineEmits<{
  set: [SettingKey, SettingValue]
  shortcuts: [Event]
  touchEdit: [Event]
}>()

const { t } = useI18n()
const shortcutsMarks = { [FocusMark.Shortcuts]: '', [FocusMark.SubInitial]: '' }
const touchEditMarks = { [FocusMark.TouchEdit]: '' }
</script>

<template>
  <MenuList>
    <Card :variant="CardVariant.ListRow" class="gap-12">
      <span class="min-w-0 flex-1">
        <span class="block font-medium">{{ t('qm.controller') }}</span>
        <span class="block text-caption text-ink-muted">
          {{ controller ?? t('qm.noController') }}
        </span>
      </span>
    </Card>
    <SessionSettingRows
      v-if="schema"
      :entries="entries"
      :rows="rows"
      :schema="schema"
      @set="(key, v) => emit('set', key, v)"
    />
    <Card
      v-if="touch"
      as="button"
      v-bind="touchEditMarks"
      :variant="CardVariant.ListRow"
      class="gap-12"
      @click="emit('touchEdit', $event)"
    >
      <span class="min-w-0 flex-1">
        <span class="block font-medium">{{ t('settings.touch.tedit.name') }}</span>
        <span class="block text-caption text-ink-muted">{{ t('settings.touch.tedit.desc') }}</span>
      </span>
      <ChevronRight :size="16" :stroke-width="1.75" class="shrink-0" aria-hidden="true" />
    </Card>
    <Card
      as="button"
      :variant="CardVariant.ListRow"
      class="gap-12"
      disabled
      :title="t('qm.notYet')"
    >
      <span class="min-w-0 flex-1">
        <span class="block font-medium">{{ t('qm.remap') }}</span>
        <span class="block text-caption text-ink-muted">{{ t('qm.notYet') }}</span>
      </span>
      <ChevronRight :size="16" :stroke-width="1.75" class="shrink-0" aria-hidden="true" />
    </Card>
    <Card
      as="button"
      v-bind="shortcutsMarks"
      :variant="CardVariant.ListRow"
      class="gap-12"
      @click="emit('shortcuts', $event)"
    >
      <span class="min-w-0 flex-1">
        <span class="block font-medium">{{ t('qm.shortcuts') }}</span>
        <span class="block text-caption text-ink-muted">
          {{ t('qm.shortcutsHint', { modifier: posName('generic', modifier, false) }) }}
        </span>
      </span>
      <span class="text-ink-muted tabular-nums">{{ shortcutCount }}</span>
      <ChevronRight :size="16" :stroke-width="1.75" class="shrink-0" aria-hidden="true" />
    </Card>
  </MenuList>
</template>
