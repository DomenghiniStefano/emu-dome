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
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Info } from '@lucide/vue'
import type { SettingEntry, SettingRecent, SettingsSchema } from '../emu'
import { settingsFocus } from '../gameDetails'
import { loadSchema, lockReason, msg } from '../settingsModel'
import SettingRowView from './SettingRow.vue'
import SettingsCategories from './settings/SettingsCategories.vue'
import SettingsOverview from './settings/SettingsOverview.vue'
import SettingsScopeBar from './settings/SettingsScopeBar.vue'
import SettingsSection from './settings/SettingsSection.vue'
import {
  INTROS,
  OVERVIEW,
  RESULTS,
  changedCount,
  pageSections,
  searchSections,
  shownOn,
} from './settings/settingsPage'
import { useSettingsScope } from './settings/useSettingsScope'
import { useSettingsValues } from './settings/useSettingsValues'

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
const store = useSettingsValues(schema, byKey, scope, text => emit('error', text))
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

let unlisten: (() => void) | null = null
onMounted(async () => {
  schema.value = await loadSchema()
  await scopeBar.findContextConsole()
  await store.loadAll()
  await store.loadRecent()
  unlisten = await store.follow()
})
onBeforeUnmount(() => unlisten?.())
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

    <main
      class="flex min-w-0 flex-1 flex-col gap-20 overflow-y-auto px-32 pt-24 pb-48 *:w-full *:max-w-page"
    >
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
          <h2 class="m-0 text-title">{{ t(msg.page(page)) }}</h2>
          <div class="text-ink-muted">{{ t(msg.summary(page)) }}</div>
        </div>
        <div
          v-if="INTROS.has(page)"
          class="flex gap-10 rounded-10 border border-line bg-surface px-14 py-12"
        >
          <!-- 1px down to sit on the text's first line -->
          <Info
            class="mt-[1px] flex-none text-accent-text"
            :size="16"
            :stroke-width="1.75"
            aria-hidden="true"
          />
          <span>{{ t(`settingsUi.intro.${page}`) }}</span>
        </div>

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
      <div v-else class="flex flex-col gap-4">
        <h2 class="m-0 text-heading">
          {{ t('settingsUi.resultsTitle', { query: query.trim() }) }}
        </h2>
      </div>

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
          lock-note
          @set="v => store.set(e.key, v)"
          @reset="store.reset(e.key)"
          @error="text => emit('error', text)"
          @open="emit('openControls')"
          @choose="store.chooseBackground"
        />
      </SettingsSection>

      <div
        v-if="searching && sections.length === 0"
        class="rounded-10 border border-line bg-surface p-32 text-center text-ink-muted"
      >
        {{ t('settingsUi.noResults', { query: query.trim() }) }}
      </div>
      <div
        v-else-if="onlyChanged && sections.length === 0"
        class="rounded-10 border border-line bg-surface p-32 text-center text-ink-muted"
      >
        {{ t('settingsUi.emptyOnly') }}
      </div>

      <div v-if="page !== OVERVIEW || searching" class="text-caption text-ink-muted">
        {{ t('settingsUi.hint') }}
      </div>
    </main>
  </div>
</template>
