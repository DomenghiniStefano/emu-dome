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
// Controller: the shell's nav scope; L1 / R1 change level (this section's own `tab`, nav.ts
// `useNavSection`).
// TV mode (#30): the same state and structure with the TV sizes (tv/TvSettings: the pages on the
// left, large rows on the right, ◀ ▶ change a value in place).
import { computed, nextTick, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Info } from '@lucide/vue'
import type {
  ConsoleId,
  SettingEntry,
  SettingKey,
  SettingRecent,
  SettingsSchema,
  SettingsScope,
} from '@/emu'
import LicencesPage from '@/components/about/LicencesPage.vue'
import ScreenEditor from '@/components/screen-editor/ScreenEditor.vue'
import { EVERY_SETTING, useSettingsChanged } from '@/composables/useSettingsChanged'
import { uiMode } from '@/composables/useUiMode'
import type { GameFocus } from '@/lib/gameDetails'
import { useNavSection } from '@/nav'
import { useErrorSink } from '@/composables/useToasts'
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
import TvSettings from '@/components/settings/tv/TvSettings.vue'
import { Heading, HeadingVariant } from '@/components/ui/heading'
import { EmptyState, EmptyStateVariant } from '@/components/ui/empty-state'
import { Card, CardPad } from '@/components/ui/card'
import { SectionPage, SectionWidth } from '@/components/ui/section-page'

const props = defineProps<{
  /** L06 «Apri tutte le impostazioni di questo gioco»: the game this section opened for (the
   * shell's route, read on mount). */
  game: GameFocus | null
  /** X01, the search everywhere: the setting whose page opens with its row lit (read on mount). */
  row?: SettingKey | null
}>()
const emit = defineEmits<{ openControls: [] }>()
const { t } = useI18n()

const schema = ref<SettingsSchema | null>(null)
const page = ref(OVERVIEW)
const gameFocus = props.game
const onlyChanged = ref(false)
const query = ref('')
const searching = computed(() => query.value.trim() !== '')

const byKey = computed(() => new Map((schema.value?.entries ?? []).map(e => [e.key, e])))
const pageEntries = computed(() => (schema.value ? shownOn(schema.value, page.value) : []))

const scopeBar = useSettingsScope(pageEntries, gameFocus)
const { effective, scope, levelName } = scopeBar
const store = useSettingsValues(schema, byKey, scope, useErrorSink())
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

const root = ref<HTMLElement | null>(null)
useNavSection(root, { tab: scopeBar.tab })

// `screens.custom` (C09): the screens editor over the whole window, at the page's scope; closed,
// the focus goes back to the row's button.
const editing = ref<{ scope: SettingsScope; console: ConsoleId } | null>(null)
let editOpener: HTMLElement | null = null
function editScreens() {
  const s = scope.value
  if (!('console' in s)) return
  editOpener = document.activeElement instanceof HTMLElement ? document.activeElement : null
  editing.value = { scope: s, console: s.console }
}
function closeEditor() {
  editing.value = null
  void nextTick(() => editOpener?.focus())
}

// `about.lic` (Trello #44): the licences page in the page's place; closed, the focus goes back
// to the row's button. Another row's `choose` is the background image's picker.
const licences = ref(false)
let licencesOpener: HTMLElement | null = null
function choose(key: SettingKey) {
  if (key !== 'about.lic') return store.chooseBackground()
  licencesOpener = document.activeElement instanceof HTMLElement ? document.activeElement : null
  licences.value = true
}
function closeLicences() {
  licences.value = false
  void nextTick(() => licencesOpener?.focus())
}

// X01: the row the search opened this section for, lit for `duration-highlight` (its animation
// ends it) and focused, on its page.
const lit = ref<SettingKey | null>(null)
const litRow = props.row ?? null
async function lightRow() {
  const e = litRow ? byKey.value.get(litRow) : undefined
  if (!e) return
  pick(e.page)
  await nextTick()
  const el = root.value?.querySelector<HTMLElement>(`[data-setting-key="${e.key}"]`)
  if (!el) return
  lit.value = e.key
  el.scrollIntoView({ block: 'center' })
  el.querySelector<HTMLElement>('button, input, [tabindex]:not([tabindex="-1"])')?.focus({
    preventScroll: true,
  })
}

onMounted(async () => {
  schema.value = await loadSchema()
  await scopeBar.findContextConsole()
  await store.loadAll()
  await lightRow()
  await store.loadRecent()
})
useSettingsChanged(EVERY_SETTING, store.onChanged)
</script>

<template>
  <div v-if="schema" ref="root" class="flex min-h-0 flex-1">
    <TvSettings
      v-if="uiMode.tv"
      :schema="schema"
      :page="page"
      :sections="sections"
      :values="values"
      :scope-bar="scopeBar"
      :game-focus="gameFocus"
      :locked-note="lockedNote"
      :lit="lit"
      @pick="pick"
      @set="store.set"
      @reset="store.reset"
      @open-controls="emit('openControls')"
      @choose="store.chooseBackground"
      @edit="editScreens"
      @unlit="lit = null"
      @level="l => (scopeBar.level.value = l)"
    />
    <SettingsCategories
      v-if="!uiMode.tv"
      v-model:query="query"
      :schema="schema"
      :page="page"
      :changed-on="p => changedCount(schema!, p, values[p], effective)"
      @pick="pick"
    />

    <SectionPage v-if="!uiMode.tv" as="main" :width="SectionWidth.Page">
      <LicencesPage v-if="licences" @close="closeLicences" />
      <SettingsOverview
        v-else-if="page === OVERVIEW && !searching"
        :schema="schema"
        :values="values"
        :recent="store.recent.value"
        :level-name="levelName"
        @pick="pick"
        @open-recent="openRecent"
        @set="store.set"
        @reset="store.reset"
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

      <template v-if="!licences">
        <SettingsSection
          v-for="sec in sections"
          :key="`${sec.page}.${sec.id}`"
          :title="sec.id === RESULTS ? t(msg.page(sec.page)) : t(msg.section(sec.page, sec.id))"
        >
          <SettingRowView
            v-for="e in sec.entries"
            :key="e.key"
            :data-setting-key="e.key"
            :class="lit === e.key ? 'animate-row-highlight' : undefined"
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
            @open="emit('openControls')"
            @choose="choose(e.key)"
            @edit="editScreens"
            @animationend="lit = null"
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
      </template>
    </SectionPage>

    <ScreenEditor
      v-if="editing"
      :console="editing.console"
      :scope="editing.scope"
      :status="null"
      @close="closeEditor"
    />
  </div>
</template>
