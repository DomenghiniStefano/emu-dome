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
// The phone held upright (#29, settingsPane.ts): one column, the list of pages first; a page opens
// over it with «‹ Impostazioni», and B / Android's back return to the list. On its side, the two
// columns as on the desk.
import { computed, nextTick, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft, Info } from '@lucide/vue'
import type { ConsoleId, SettingEntry, SettingKey, SettingRecent, SettingsSchema } from '@/emu'
import LicencesPage from '@/components/about/LicencesPage.vue'
import StoragePage from '@/components/storage/StoragePage.vue'
import ScreenEditor from '@/components/screen-editor/ScreenEditor.vue'
import TouchEditor from '@/components/touch/TouchEditor.vue'
import { EVERY_SETTING, useSettingsChanged } from '@/composables/useSettingsChanged'
import { uiMode } from '@/composables/useUiMode'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import type { GameFocus } from '@/lib/gameDetails'
import { keyboardSeen, showNavKeys } from '@/lib/inputMode'
import { useNavSection } from '@/nav'
import { useErrorSink } from '@/composables/useToasts'
import { keyWords, loadSchema, lockReason, msg } from '@/lib/settingsModel'
import LayoutConsoleChooser from './LayoutConsoleChooser.vue'
import {
  chosenTarget,
  directTarget,
  editorOf,
  type EditorTarget,
  type LayoutEditor,
} from './layoutEditors'
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
import { listShown, pageShown, settingsPane, SettingsPane } from './settingsPane'
import { useSettingsScope } from './useSettingsScope'
import { useSettingsValues } from './useSettingsValues'
import TvSettings from '@/components/settings/tv/TvSettings.vue'
import { Heading, HeadingVariant } from '@/design-system/components/heading'
import { EmptyState, EmptyStateVariant } from '@/design-system/components/empty-state'
import { Card, CardPad } from '@/design-system/components/card'
import { SectionPage, SectionWidth } from '@/design-system/components/section-page'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'

const props = defineProps<{
  /** L06 «Apri tutte le impostazioni di questo gioco»: the game this section opened for (the
   * shell's route, read on mount). */
  game: GameFocus | null
  /** X01, the search everywhere: the setting whose page opens with its row lit (read on mount). */
  row?: SettingKey | null
  /** With `row`: its layout editor opened at once (C01's «Controlli touch» → `touch.tedit`,
   * #112), as its button would. */
  editRow?: boolean
}>()
const emit = defineEmits<{
  openControls: []
  openCloud: []
  /** The editor or page this section was opened for from another (`editRow`) is closed: the
   * shell goes back to where it came from (#144: C01's «Controlli touch» back to Controls). */
  leave: []
}>()
const { t, te } = useI18n()

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
    `${t(msg.name(e.key))} ${t(msg.desc(e.key))} ${keyWords(
      e.key,
      k => te(k),
      k => t(k),
    )} ${e.key} ${e.synonyms.join(' ')}`
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

// The phone held upright: one pane at a time (the list until a page is picked).
const layout = useDeviceLayout()
/** The page's foot on the keyboard and the controller: only with one of them there (#131). */
const navHint = computed(() =>
  showNavKeys(layout.touchDensity.value, keyboardSeen.value, uiMode.pad !== null),
)
const listOpen = ref(true)
const pane = computed(() =>
  settingsPane(layout.phone.value && !uiMode.tv, listOpen.value, searching.value),
)

function pick(p: string) {
  page.value = p
  query.value = ''
  onlyChanged.value = false
  listOpen.value = false
}
/** Upright, from a page: back to the list, on that page's entry. */
function showList() {
  listOpen.value = true
  void nextTick(() => root.value?.querySelector<HTMLElement>('nav [aria-current="page"]')?.focus())
}
/** A recent change opens its page, at the level it was made when the page has it. */
function openRecent(r: SettingRecent) {
  const e = byKey.value.get(r.key)
  if (!e) return
  pick(e.page)
  scopeBar.goTo(r.level, r.console)
}

const root = ref<HTMLElement | null>(null)
useNavSection(root, {
  tab: scopeBar.tab,
  // Upright on a page: B / Android's back return to the list; anywhere else the shell's back.
  back: () => (pane.value === SettingsPane.Page ? showList() : false),
})

// `screens.custom` (C09): the screens editor over the whole window, at the page's scope;
// `touch.tedit` (C06): the touch controls editor for the page's console. With no console the
// editor has layouts for (at «Tutti i giochi»), the console is chosen first (Trello #111,
// layoutEditors.ts). Closed, the focus goes back to the row's button.
const editing = ref<EditorTarget | null>(null)
const choosing = ref<LayoutEditor | null>(null)
let editOpener: HTMLElement | null = null
function editScreens(key: SettingKey = 'screens.custom') {
  editOpener = document.activeElement instanceof HTMLElement ? document.activeElement : null
  const editor = editorOf(key)
  const target = directTarget(editor, scope.value)
  if (target) editing.value = target
  else choosing.value = editor
}
function chooseConsole(console: ConsoleId) {
  const editor = choosing.value
  choosing.value = null
  if (editor) editing.value = chosenTarget(editor, console)
}
function closeEditor() {
  editing.value = null
  choosing.value = null
  if (leaveOnClose()) return
  void nextTick(() => editOpener?.focus())
}

// `about.lic` (Trello #44): the licences page in the page's place; `saves.space` (#29):
// «Gestisci lo spazio» there too. Closed, the focus goes back to the row's button. Another row's
// `choose` is the background image's picker.
type AsidePage = 'licences' | 'space'
const ASIDE_ROWS: Partial<Record<SettingKey, AsidePage>> = {
  'about.lic': 'licences',
  'saves.space': 'space',
}
const aside = ref<AsidePage | null>(null)
let asideOpener: HTMLElement | null = null
function choose(key: SettingKey) {
  const to = ASIDE_ROWS[key]
  if (!to) return store.chooseBackground()
  asideOpener = document.activeElement instanceof HTMLElement ? document.activeElement : null
  aside.value = to
}
function closeAside() {
  aside.value = null
  if (leaveOnClose()) return
  void nextTick(() => asideOpener?.focus())
}

// Opened for another section's button (`editRow`: C01's «Controlli touch», E13's «Gestisci lo
// spazio»), the editor or page goes back there when closed, once (#144); opened from this
// section's own row it stays here.
let openedFromElsewhere = false
function leaveOnClose(): boolean {
  if (!openedFromElsewhere) return false
  openedFromElsewhere = false
  emit('leave')
  return true
}

// X01: the row the search opened this section for, lit for `duration-highlight` (its animation
// ends it) and focused, on its page.
const lit = ref<SettingKey | null>(null)
const litRow = props.row ?? null
const editRow = props.editRow ?? false
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
  // E13's «Gestisci lo spazio» (#29) arrives as `saves.space` with `edit`: its page opens.
  if (litRow && editRow) {
    openedFromElsewhere = true
    if (ASIDE_ROWS[litRow]) choose(litRow)
    else editScreens(litRow)
  }
  await store.loadRecent()
})
useSettingsChanged(EVERY_SETTING, store.onChanged)
</script>

<template>
  <div
    v-if="schema"
    ref="root"
    :class="['flex min-h-0 flex-1', pane !== SettingsPane.Both && 'flex-col']"
  >
    <!-- `about.lic` and `saves.space` in TV: the desk's page, scaled up as the sections not laid
         out for TV are (AppShell), in the place of the TV rows. -->
    <div v-if="uiMode.tv && aside" class="flex min-h-0 flex-1 flex-col tv:zoom-tv">
      <SectionPage as="main" :width="SectionWidth.Full">
        <LicencesPage v-if="aside === 'licences'" @close="closeAside" />
        <StoragePage v-else @close="closeAside" />
      </SectionPage>
    </div>
    <TvSettings
      v-else-if="uiMode.tv"
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
      @open-cloud="emit('openCloud')"
      @choose="choose"
      @edit="editScreens"
      @unlit="lit = null"
      @level="l => (scopeBar.level.value = l)"
    />
    <SettingsCategories
      v-if="!uiMode.tv && listShown(pane)"
      v-model:query="query"
      :schema="schema"
      :page="page"
      :pane="pane"
      :changed-on="p => changedCount(schema!, p, values[p], effective)"
      @pick="pick"
    />

    <SectionPage v-if="!uiMode.tv && pageShown(pane)" as="main" :width="SectionWidth.Full">
      <!-- A block of the page (held to its width), the way back at its start; a page shown in
           the page's place (licences, «Gestisci lo spazio») has its own. -->
      <div v-if="pane === SettingsPane.Page && !aside" class="flex">
        <Button
          :variant="ButtonVariant.Ghost"
          :size="ButtonSize.TextLine"
          class="-ml-6"
          :icon="ChevronLeft"
          data-nav-initial
          @click="showList"
        >
          {{ t('settingsUi.title') }}
        </Button>
      </div>
      <LicencesPage v-if="aside === 'licences'" @close="closeAside" />
      <StoragePage v-else-if="aside === 'space'" @close="closeAside" />
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
        <Card v-if="INTROS.has(page)" :pad="CardPad.Snug" class="flex-row gap-8">
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

      <template v-if="!aside">
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
            @open-cloud="emit('openCloud')"
            @choose="choose(e.key)"
            @edit="editScreens(e.key)"
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

        <div v-if="navHint && (page !== OVERVIEW || searching)" class="text-caption text-ink-muted">
          {{ t('settingsUi.hint') }}
        </div>
      </template>
    </SectionPage>

    <TouchEditor
      v-if="editing?.editor === 'touch'"
      :console="editing.console"
      :live="false"
      @close="closeEditor"
    />
    <ScreenEditor
      v-else-if="editing"
      :console="editing.console"
      :scope="editing.scope"
      :status="null"
      @close="closeEditor"
    />
    <LayoutConsoleChooser
      v-if="choosing"
      :editor="choosing"
      @choose="chooseConsole"
      @cancel="closeEditor"
    />
  </div>
</template>
