<script setup lang="ts">
// T01 «In primo piano» (board Settings2, 17/11): «Usate più spesso», the entries flagged
// `mostUsed` in the schema, as rows at «Tutti i giochi» (a cards choice does not fit a row: its
// current option, as a link to its page); «Modificate di recente» (`settings_recent`, newest
// first), each opening its page at the level it was made.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type {
  SettingEntry,
  SettingKey,
  SettingLevel,
  SettingRecent,
  SettingsSchema,
  SettingValue,
} from '@/emu'
import { changedWhenLabel } from '@/i18n/labels'
import { isShown, msg } from '@/lib/settingsModel'
import SettingRowView from './SettingRow.vue'
import SettingsSection from './SettingsSection.vue'
import type { PageValues } from './settingsPage'
import { Heading } from '@/components/ui/heading'
import { LinkRow } from '@/components/ui/link-row'

const props = defineProps<{
  schema: SettingsSchema
  values: Record<string, PageValues>
  recent: SettingRecent[]
  levelName: (level: SettingLevel) => string
}>()
const emit = defineEmits<{
  pick: [page: string]
  openRecent: [SettingRecent]
  set: [key: SettingKey, value: SettingValue]
  reset: [key: SettingKey]
  error: [string]
}>()
const { t } = useI18n()

const mostUsed = computed(() =>
  props.schema.entries.filter(e => e.mostUsed && isShown(e, props.schema)),
)
const isCards = (e: SettingEntry) => e.kind.type === 'choice' && e.kind.style === 'cards'
const storedValue = (e: SettingEntry) => props.values[e.page]?.[e.key]?.value

/** The current option of a cards entry («Effetto schermo»). */
function optionText(e: SettingEntry): string {
  const v = storedValue(e) ?? e.default
  return typeof v === 'string' ? t(msg.option(e.key, v)) : ''
}

/** «Controller e tastiera · Come stampata sul controller · ieri»: page, then the value set at
 * «Tutti i giochi» or the level it was set at, then when. */
function recentLine(r: SettingRecent): string {
  const e = props.schema.entries.find(x => x.key === r.key)
  if (!e) return ''
  const parts = [t(msg.page(e.page))]
  if (r.level === 'console' && r.console)
    parts.push(t('settingsUi.scope.console', { console: t(`console.${r.console}`) }))
  else if (r.level === 'game') parts.push(t('controls.levelGame'))
  else {
    const v = storedValue(e)
    if (e.kind.type === 'choice' && typeof v === 'string') parts.push(t(msg.option(e.key, v)))
    else if (e.kind.type === 'slider' && typeof v === 'number')
      parts.push(t(`settingsUi.unit.${e.kind.unit}`, { v }))
  }
  parts.push(changedWhenLabel(r.at))
  return parts.join(' · ')
}
</script>

<template>
  <Heading>{{ t('settingsUi.overview.name') }}</Heading>
  <SettingsSection
    :title="t('settingsUi.overview.mostUsed')"
    :note="t('settingsUi.overview.mostUsedNote')"
  >
    <template v-for="e in mostUsed" :key="e.key">
      <LinkRow
        v-if="isCards(e)"
        :name="t(msg.name(e.key))"
        :sub="t(msg.page(e.page))"
        :value="optionText(e)"
        @click="emit('pick', e.page)"
      />
      <SettingRowView
        v-else
        :entry="e"
        :row="values[e.page]?.[e.key]"
        :schema="schema"
        level="global"
        :level-name="levelName"
        :scoped="false"
        compact
        :sub="t(msg.page(e.page))"
        @set="v => emit('set', e.key, v)"
        @reset="emit('reset', e.key)"
        @error="text => emit('error', text)"
      />
    </template>
  </SettingsSection>

  <SettingsSection v-if="recent.length" :title="t('settingsUi.overview.recent')">
    <LinkRow
      v-for="r in recent"
      :key="`${r.key}.${r.level}.${r.console ?? ''}.${r.game ?? ''}`"
      :name="t(msg.name(r.key))"
      :sub="recentLine(r)"
      @click="emit('openRecent', r)"
    />
  </SettingsSection>
</template>
