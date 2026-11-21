<script setup lang="ts">
// The Settings section (boards «T01–T19 · Desktop · Impostazioni», ScopeSelector, InheritedRow):
// the category list with the title, the search and changed counters on the left
// (SettingsCategories), the page on the right with the scope bar fixed at the top while scrolling
// (SettingsScopeBar). It opens on T01 «In primo piano» (SettingsOverview).
// Every page is built from the Rust schema (`settings_schema`): groups, sections, entries, and
// which rows show (`showIf`, against the values at the level being edited). Rows that only
// «Tutti i giochi» can change are greyed at other levels, with one note after the scope sentence.
// The level edited is useSettingsScope's, the values useSettingsValues', the choice of rows
// settingsPage.ts's.
// Controller: the shell's nav scope; L1 / R1 change level (exposed `tab`, called by AppShell).
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Info } from '@lucide/vue'
import type { SettingEntry, SettingRecent, SettingsSchema } from '@/emu'
import { EVERY_SETTING, useSettingsChanged } from '@/composables/useSettingsChanged'
import { settingsFocus } from '@/lib/gameDetails'
import { errorText } from '@/i18n/labels'
import { loadSchema, lockReason, msg } from '@/lib/settingsModel'
import SettingRowView from './SettingRow.vue'
import SettingsCategories from './SettingsCategories.vue'
import SettingsOverview from './SettingsOverview.vue'
import SettingsScopeBar from './SettingsScopeBar.vue'
import SettingsSection from './SettingsSection.vue'
import {
  INTROS,
  OVERVIEW,
  RESULTS,
  changedCount,
  pageSections,
  searchSections,
  shownOn,
} from './settingsPage'
import { useSettingsScope } from './useSettingsScope'
import { useSettingsValues } from './useSettingsValues'
import { Heading, HeadingVariant } from '@/components/ui/heading'
import { EmptyState, EmptyStateVariant } from '@/components/ui/empty-state'
import { Card, CardPad } from '@/components/ui/card'
import { SectionPage, SectionWidth } from '@/components/ui/section-page'

const emit = defineEmits<{ error: [string]; openControls: [] }>()
const { t } = useI18n()

const schema = ref<SettingsSchema | null>(null)
const page = ref(OVERVIEW)
/** L06 «Apri tutte le impostazioni di questo gioco»: the game this section opened for. */
const gameFocus = settingsFocus.value
settingsFocus.value = null
const onlyChanged = ref(false)
const query = ref('')
const searching = computed(() => query.value.trim() !== '')

const byKey = computed(() => new Map((schema.value?.entries ?? []).map(e => [e.key, e])))
const pageEntries = computed(() => (schema.value ? shownOn(schema.value, page.value) : []))

const scopeBar = useSettingsScope(pageEntries, gameFocus)
const { effective, scope, levelName } = scopeBar
const store = useSettingsValues(schema, byKey, scope, e => emit('error', errorText(e)))
const { values } = store

const sections = computed(() => {
  const s = schema.value
  if (!s) return []
  const words = (e: SettingEntry) =>
    `${t(msg.name(e.key))} ${t(msg.desc(e.key))} ${e.key} ${e.synonyms.join(' ')}`
  if (searching.value) return searchSections(s, query.value, values, byKey.value, words)
  const onlyAt = onlyChanged.value ? effective.value : null
  return pageSections(s, page.value, values, byKey.value, onlyAt)
})

/** Rows of the page that only «Tutti i giochi» changes, greyed here: the note says it once. */
const lockedNote = computed(() => {
  const s = schema.value
  if (!s || effective.value === 'global') return ''
  const any = sections.value.some(sec =>
    sec.entries.some(e => lockReason(e, s, effective.value) === 'settingsUi.lockedGlobal'),
  )
  return any ? t('settingsUi.lockedNote') : ''
})

function pick(p: string) {
  page.value = p
  query.value = ''
  onlyChanged.value = false
}
/** A recent change opens its page, at the level it was made when the page has it. */
function openRecent(r: SettingRecent) {
  const e = byKey.value.get(r.key)
  if (!e) return
  pick(e.page)
  scopeBar.goTo(r.level, r.console)
}

defineExpose({ tab: scopeBar.tab })

onMounted(async () => {
  schema.value = await loadSchema()
  await scopeBar.findContextConsole()
  await store.loadAll()
  await store.loadRecent()
})
useSettingsChanged(EVERY_SETTING, store.onChanged)
</script>

<template>
  <div v-if="schema" class="flex min-h-0 flex-1">
    <SettingsCategories
      v-model:query="query"
      :schema="schema"
      :page="page"
      :changed-on="p => changedCount(schema!, p, values[p], effective)"
      @pick="pick"
    />

    <SectionPage as="main" :width="SectionWidth.Page">
      <SettingsOverview
        v-if="page === OVERVIEW && !searching"
        :schema="schema"
        :values="values"
        :recent="store.recent.value"
        :level-name="levelName"
        @pick="pick"
        @open-recent="openRecent"
        @set="store.set"
        @reset="store.reset"
        @error="text => emit('error', text)"
      />

      <template v-else-if="!searching">
        <div class="flex flex-col gap-4">
          <Heading>{{ t(msg.page(page)) }}</Heading>
          <div class="text-ink-muted">{{ t(msg.summary(page)) }}</div>
        </div>
        <Card v-if="INTROS.has(page)" :pad="CardPad.Snug" class="flex-row gap-10">
          <!-- 1px down to sit on the text's first line -->
          <Info
            class="mt-px flex-none text-accent-text"
            :size="16"
            :stroke-width="1.75"
            aria-hidden="true"
          />
          <span>{{ t(`settingsUi.intro.${page}`) }}</span>
        </Card>

        <SettingsScopeBar
          v-if="scopeBar.scoped.value"
          v-model:only-changed="onlyChanged"
          :levels="scopeBar.levels.value"
          :effective="effective"
          :context-console="scopeBar.contextConsole.value"
          :console-options="scopeBar.consoleOptions.value"
          :game-focus="gameFocus"
          :level-name="levelName"
          :locked-note="lockedNote"
          @level="l => (scopeBar.level.value = l)"
          @pick-console="scopeBar.pickConsole"
        />
        <div v-else class="text-ink-muted">{{ t('settingsUi.appWide') }}</div>
      </template>
      <Heading v-else :variant="HeadingVariant.Heading">
        {{ t('settingsUi.resultsTitle', { query: query.trim() }) }}
      </Heading>

      <SettingsSection
        v-for="sec in sections"
        :key="`${sec.page}.${sec.id}`"
        :title="sec.id === RESULTS ? t(msg.page(sec.page)) : t(msg.section(sec.page, sec.id))"
      >
        <SettingRowView
          v-for="e in sec.entries"
          :key="e.key"
          :entry="e"
          :row="values[sec.page]?.[e.key]"
          :schema="schema"
          :level="effective"
          :level-name="levelName"
          :scoped="scopeBar.scoped.value"
          :console="'console' in scope ? scope.console : null"
          lock-note
          @set="v => store.set(e.key, v)"
          @reset="store.reset(e.key)"
          @error="text => emit('error', text)"
          @open="emit('openControls')"
          @choose="store.chooseBackground"
        />
      </SettingsSection>

      <EmptyState
        v-if="searching && sections.length === 0"
        as="div"
        :variant="EmptyStateVariant.Card"
        :text="t('settingsUi.noResults', { query: query.trim() })"
      />
      <EmptyState
        v-else-if="onlyChanged && sections.length === 0"
        as="div"
        :variant="EmptyStateVariant.Card"
        :text="t('settingsUi.emptyOnly')"
      />

      <div v-if="page !== OVERVIEW || searching" class="text-caption text-ink-muted">
        {{ t('settingsUi.hint') }}
      </div>
    </SectionPage>
  </div>
</template>
