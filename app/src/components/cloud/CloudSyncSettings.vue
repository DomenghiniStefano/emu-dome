<script setup lang="ts">
// The tab «Cosa sincronizzare» (Trello #31): the `cloud.*` settings from the schema, with the
// Settings section's own rows (SettingRow), at «Tutti i giochi». Settings of one device never
// sync: the rows say so where it matters (`cloud.ssettings`).
import { useI18n } from 'vue-i18n'
import { useErrorSink } from '@/composables/useToasts'
import { msg } from '@/lib/settingsModel'
import SettingRowView from '@/components/settings/SettingRow.vue'
import SettingsSection from '@/components/settings/SettingsSection.vue'
import { useCloudSettings } from './useCloudSettings'

const emit = defineEmits<{ openGames: [] }>()
const { t } = useI18n()
const store = useCloudSettings(useErrorSink())
const levelName = () => t('settingsUi.scope.global')
</script>

<template>
  <template v-if="store.schema.value">
    <SettingsSection
      v-for="sec in store.sections.value"
      :key="sec.id"
      :title="t(msg.section(sec.page, sec.id))"
    >
      <SettingRowView
        v-for="e in sec.entries"
        :key="e.key"
        :entry="e"
        :row="store.rows.value[e.key]"
        :schema="store.schema.value"
        level="global"
        :level-name="levelName"
        :scoped="false"
        @set="v => store.set(e.key, v)"
        @reset="store.reset(e.key)"
        @open="emit('openGames')"
      />
    </SettingsSection>
  </template>
</template>
