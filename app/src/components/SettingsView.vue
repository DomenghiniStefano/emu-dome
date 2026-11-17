<script setup lang="ts">
// The Settings section (boards «T01–T20 · Desktop · Impostazioni», ScopeSelector, InheritedRow):
// category list with search and changed counters on the left, the page on the right with the
// scope bar «Stai cambiando le impostazioni di:» fixed at the top while scrolling.
// Every page is built from the Rust schema (`settings_schema`): groups, sections, entries.
// The section opens on T01 «In primo piano» (board Settings2, 17/11), first in the category list:
// the search, «Usate più spesso» (the entries flagged `mostUsed` in the schema, inline at
// «Tutti i giochi») and «Modificate di recente» (`settings_recent`, newest first).
//
// Levels: «Tutti i giochi», and «Giochi <console> ▾», a small menu of the consoles that have games
// in the library, starting on the open game's console, else the last one played (Cowork, #41).
// With an empty library only «Tutti i giochi». «Solo questo gioco» belongs to the game details
// (L04) and the quick menu (Session), not to this section.
// Controller: the shell's nav scope; L1 / R1 change level (exposed `tab`, called by AppShell).
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronRight, Gamepad2, Info, Layers, Search } from '@lucide/vue'
import {
  emu,
  type ConsoleId,
  type SettingEntry,
  type SettingRecent,
  type SettingLevel,
  type SettingRow,
  type SettingsSchema,
  type SettingsScope,
  type SettingValue,
} from '../emu'
import { changedWhenLabel, errorText } from '../i18n/labels'
import { library } from '../library'
import { fold, isShown, loadSchema, msg, pageLevels } from '../settingsModel'
import SettingRowView from './SettingRow.vue'
import SettingSelect from './SettingSelect.vue'

const emit = defineEmits<{ error: [string]; openControls: [] }>()
const { t } = useI18n()

const schema = ref<SettingsSchema | null>(null)
/** T01 «In primo piano»: not a page of the schema, the section's first screen. */
const OVERVIEW = 'overview'
const page = ref(OVERVIEW)
const level = ref<'global' | 'console'>('global')
const onlyChanged = ref(false)
const query = ref('')
/** Values of every page at the current scope, by page then key. */
const values = reactive<Record<string, Record<string, SettingRow>>>({})

const byKey = computed(() => {
  const m = new Map<string, SettingEntry>()
  for (const e of schema.value?.entries ?? []) m.set(e.key, e)
  return m
})
const shownOn = (p: string) =>
  (schema.value?.entries ?? []).filter(e => e.page === p && isShown(e, schema.value!))

// ---- Scope ----
const CONSOLES: ConsoleId[] = ['gb', 'gbc', 'gba', 'nds', '3ds']
const contextConsole = ref<ConsoleId | null>(null)
/** The open game's console, when a game is open. */
const openConsole = ref<ConsoleId | null>(null)
/** The consoles of the scope menu: those with games in the library (and the open game's). */
const consoles = computed(() =>
  CONSOLES.filter(c => c === openConsole.value || library.games.some(g => g.console === c)),
)
const consoleOptions = computed(() =>
  consoles.value.map(c => ({
    id: c,
    text: t('settingsUi.scope.console', { console: t(`console.${c}`) }),
  })),
)
function lastPlayedConsole(): ConsoleId | null {
  const last = [...library.games]
    .filter(g => g.lastPlayed !== null)
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))[0]
  return last?.console ?? null
}
async function findContextConsole() {
  const open = await emu.settings.gameScope().catch(() => null)
  openConsole.value = open && open.level === 'game' ? open.console : null
  contextConsole.value = openConsole.value ?? lastPlayedConsole() ?? consoles.value[0] ?? null
}
// The library may answer after this section opened, or lose the chosen console's last game.
watch(consoles, list => {
  if (contextConsole.value && list.includes(contextConsole.value)) return
  contextConsole.value = lastPlayedConsole() ?? list[0] ?? null
})
function pickConsole(id: string) {
  const c = consoles.value.find(x => x === id)
  if (!c) return
  contextConsole.value = c
  level.value = 'console'
}

const pageEntries = computed(() => shownOn(page.value))
/** The page has settings per console: the scope bar is shown (only «Tutti i giochi» without games). */
const scoped = computed(() => pageLevels(pageEntries.value).has('console'))
const levels = computed<SettingLevel[]>(() => {
  const out: SettingLevel[] = ['global']
  if (scoped.value && contextConsole.value) out.push('console')
  return out
})
const effective = computed<'global' | 'console'>(() =>
  level.value === 'console' && levels.value.includes('console') ? 'console' : 'global',
)
const scope = computed<SettingsScope>(() =>
  effective.value === 'console' && contextConsole.value
    ? { level: 'console', console: contextConsole.value }
    : { level: 'global' },
)
function levelName(l: SettingLevel): string {
  if (l === 'console' && contextConsole.value)
    return t('settingsUi.scope.console', { console: t(`console.${contextConsole.value}`) })
  return t('settingsUi.scope.global')
}

// ---- Values ----
async function loadPage(p: string) {
  try {
    const rows = await emu.settings.page(p, scope.value)
    values[p] = Object.fromEntries(rows.map(r => [r.key, r]))
  } catch (e) {
    emit('error', errorText(e))
  }
}
async function loadAll() {
  if (!schema.value) return
  await Promise.all(schema.value.pages.map(loadPage))
}
watch(scope, () => void loadAll(), { deep: true })

async function set(key: string, value: SettingValue) {
  try {
    await emu.settings.set(key, scope.value, value)
  } catch (e) {
    emit('error', errorText(e))
  }
}
async function reset(key: string) {
  try {
    await emu.settings.reset(key, scope.value)
  } catch (e) {
    emit('error', errorText(e))
  }
}

/** Settings set at the current level, per page (the counters of the category list). */
function changedCount(p: string): number {
  const rows = values[p]
  if (!rows) return 0
  return shownOn(p).filter(e => rows[e.key]?.source === effective.value).length
}

// ---- Page content ----
interface ShownSection {
  id: string
  page: string
  entries: SettingEntry[]
}
const sections = computed<ShownSection[]>(() => {
  const s = schema.value
  if (!s) return []
  if (query.value.trim()) return searchResults.value
  const rows = values[page.value] ?? {}
  return s.sections
    .filter(sec => sec.page === page.value)
    .map(sec => ({
      id: sec.id,
      page: sec.page,
      entries: sec.keys
        .map(k => byKey.value.get(k))
        .filter((e): e is SettingEntry => !!e && isShown(e, s))
        .filter(e => !onlyChanged.value || rows[e.key]?.source === effective.value),
    }))
    .filter(sec => sec.entries.length > 0)
})

// ---- Search: names and descriptions in the current language, across every page ----
const searchResults = computed<ShownSection[]>(() => {
  const s = schema.value
  const q = fold(query.value.trim())
  if (!s || !q) return []
  const out: ShownSection[] = []
  for (const p of s.pages) {
    const hits = shownOn(p).filter(e =>
      fold(`${t(msg.name(e.key))} ${t(msg.desc(e.key))} ${e.key} ${e.synonyms.join(' ')}`).includes(
        q,
      ),
    )
    if (hits.length) out.push({ id: 'results', page: p, entries: hits })
  }
  return out
})

function pick(p: string) {
  page.value = p
  query.value = ''
  onlyChanged.value = false
}

// ---- T01 «In primo piano» ----
const mostUsed = computed(() =>
  (schema.value?.entries ?? []).filter(e => e.mostUsed && isShown(e, schema.value!)),
)
/** The current option of a cards entry («Effetto schermo»), shown as a link to its page. */
function optionText(e: SettingEntry): string {
  const v = values[e.page]?.[e.key]?.value ?? e.default
  return typeof v === 'string' ? t(msg.option(e.key, v)) : ''
}

const RECENT = 5
const recent = ref<SettingRecent[]>([])
async function loadRecent() {
  try {
    recent.value = (await emu.settings.recent(RECENT)).filter(r => {
      const e = byKey.value.get(r.key)
      return !!e && !!schema.value && isShown(e, schema.value)
    })
  } catch (e) {
    emit('error', errorText(e))
  }
}
/** «Controller e tastiera · Come stampata sul controller · ieri»: page, then the value set at
 * «Tutti i giochi» or the level it was set at, then when. */
function recentLine(r: SettingRecent): string {
  const e = byKey.value.get(r.key)
  if (!e) return ''
  const parts = [t(msg.page(e.page))]
  if (r.level === 'console' && r.console)
    parts.push(t('settingsUi.scope.console', { console: t(`console.${r.console}`) }))
  else if (r.level === 'game') parts.push(t('controls.levelGame'))
  else {
    const v = values[e.page]?.[e.key]?.value
    if (e.kind.type === 'choice' && typeof v === 'string') parts.push(t(msg.option(e.key, v)))
    else if (e.kind.type === 'slider' && typeof v === 'number')
      parts.push(t(`settingsUi.unit.${e.kind.unit}`, { v }))
  }
  parts.push(changedWhenLabel(r.at))
  return parts.join(' · ')
}
/** A recent change opens its page, at the level it was made when the page has it. */
function openRecent(r: SettingRecent) {
  const e = byKey.value.get(r.key)
  if (!e) return
  pick(e.page)
  if (r.level === 'console' && r.console && consoles.value.includes(r.console)) {
    contextConsole.value = r.console
    level.value = 'console'
  } else level.value = 'global'
}

/** L1 / R1: the previous / next level of the scope bar. */
function tab(dir: -1 | 1) {
  if (levels.value.length < 2) return
  const i = levels.value.indexOf(effective.value)
  const next = levels.value[(i + dir + levels.value.length) % levels.value.length]
  if (next === 'global' || next === 'console') level.value = next
}
defineExpose({ tab })

let unlisten: (() => void) | null = null
onMounted(async () => {
  schema.value = await loadSchema()
  await findContextConsole()
  await loadAll()
  await loadRecent()
  unlisten = await emu.settings.onChanged(e => {
    const p = e.key.split('.')[0]
    if (p) void loadPage(p)
    void loadRecent()
  })
})
onBeforeUnmount(() => unlisten?.())

const INTROS = new Set(['bios', 'cores'])
</script>

<template>
  <div v-if="schema" class="settings">
    <nav class="cats" :aria-label="t('settingsUi.categories')">
      <button
        class="cat overview"
        :class="{ on: page === OVERVIEW && !query }"
        :aria-current="page === OVERVIEW && !query ? 'page' : undefined"
        :data-nav-initial="page === OVERVIEW ? '' : undefined"
        @click="pick(OVERVIEW)"
      >
        <span class="cat-name">{{ t('settingsUi.overview.name') }}</span>
      </button>
      <div v-for="g in schema.groups" :key="g.id" class="group">
        <div class="overline">{{ t(msg.group(g.id)) }}</div>
        <button
          v-for="p in g.pages"
          :key="p"
          class="cat"
          :class="{ on: p === page && !query }"
          :aria-current="p === page && !query ? 'page' : undefined"
          :data-nav-initial="p === page ? '' : undefined"
          @click="pick(p)"
        >
          <span class="cat-name">{{ t(msg.page(p)) }}</span>
          <span v-if="changedCount(p)" class="count num">{{ changedCount(p) }}</span>
        </button>
      </div>
    </nav>

    <main class="page">
      <!-- T01: the title and the search; the search stays on top while showing results -->
      <template v-if="page === OVERVIEW || query.trim()">
        <h1 class="page-title">{{ t('settingsUi.title') }}</h1>
        <label class="search">
          <Search :size="16" :stroke-width="1.75" aria-hidden="true" />
          <span class="sr">{{ t('settingsUi.search') }}</span>
          <input v-model="query" type="search" :placeholder="t('settingsUi.overview.search')" />
        </label>
      </template>

      <template v-if="page === OVERVIEW && !query.trim()">
        <section class="section">
          <div class="section-head">
            <h3 class="overline">{{ t('settingsUi.overview.mostUsed') }}</h3>
            <span class="note">{{ t('settingsUi.overview.mostUsedNote') }}</span>
          </div>
          <div class="rows">
            <template v-for="e in mostUsed" :key="e.key">
              <!-- Cards do not fit a row: the current choice, as a link to its page -->
              <button
                v-if="e.kind.type === 'choice' && e.kind.style === 'cards'"
                class="link-row"
                @click="pick(e.page)"
              >
                <span class="link-text">
                  <span class="link-name">{{ t(msg.name(e.key)) }}</span>
                  <span class="link-sub">{{ t(msg.page(e.page)) }}</span>
                </span>
                <span class="link-value">{{ optionText(e) }}</span>
                <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
              </button>
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
                @set="v => set(e.key, v)"
                @reset="reset(e.key)"
                @error="text => emit('error', text)"
              />
            </template>
          </div>
        </section>

        <section v-if="recent.length" class="section">
          <h3 class="overline">{{ t('settingsUi.overview.recent') }}</h3>
          <div class="rows">
            <button
              v-for="r in recent"
              :key="`${r.key}.${r.level}.${r.console ?? ''}.${r.game ?? ''}`"
              class="link-row"
              @click="openRecent(r)"
            >
              <span class="link-text">
                <span class="link-name">{{ t(msg.name(r.key)) }}</span>
                <span class="link-sub">{{ recentLine(r) }}</span>
              </span>
              <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
            </button>
          </div>
        </section>
      </template>

      <template v-else-if="!query.trim()">
        <div class="page-head">
          <h2 class="page-title">{{ t(msg.page(page)) }}</h2>
          <div class="muted">{{ t(msg.summary(page)) }}</div>
        </div>
        <div v-if="INTROS.has(page)" class="intro">
          <Info :size="16" :stroke-width="1.75" aria-hidden="true" />
          <span>{{ t(`settingsUi.intro.${page}`) }}</span>
        </div>

        <div
          v-if="scoped"
          class="scope glass-thick"
          role="region"
          :aria-label="t('settingsUi.scopeRegion')"
        >
          <div class="scope-head">
            <div class="scope-title">{{ t('settingsUi.scopeTitle') }}</div>
            <button
              class="only"
              role="switch"
              :aria-checked="onlyChanged"
              @click="onlyChanged = !onlyChanged"
            >
              {{ t('settingsUi.onlyChanged') }}
              <span class="switch" :class="{ on: onlyChanged }"><span class="knob"></span></span>
            </button>
          </div>
          <div
            class="levels"
            role="radiogroup"
            :aria-label="t('settingsUi.scopeTitle')"
            :style="{ gridTemplateColumns: `repeat(${levels.length}, minmax(0, 1fr))` }"
          >
            <button
              class="level"
              :class="{ on: effective === 'global' }"
              role="radio"
              :aria-checked="effective === 'global'"
              @click="level = 'global'"
            >
              <Layers :size="20" :stroke-width="1.75" aria-hidden="true" />
              <span class="level-text">
                <span class="level-name">{{ levelName('global') }}</span>
                <span class="level-sub">{{ t('settingsUi.scope.globalSub') }}</span>
              </span>
            </button>
            <!-- «Giochi <console> ▾»: picking a console also selects this level -->
            <div
              v-if="levels.includes('console') && contextConsole"
              class="level"
              :class="{ on: effective === 'console' }"
              role="radio"
              :aria-checked="effective === 'console'"
              @click="level = 'console'"
            >
              <Gamepad2 :size="20" :stroke-width="1.75" aria-hidden="true" />
              <span class="level-text">
                <SettingSelect
                  :label="t('settingsUi.scope.consolePick')"
                  :value="contextConsole"
                  :options="consoleOptions"
                  @pick="pickConsole"
                  @click.stop
                />
                <span class="level-sub">
                  {{
                    t('settingsUi.scope.consoleSub', {
                      short: t(`consoleShort.${contextConsole}`),
                    })
                  }}
                </span>
              </span>
            </div>
          </div>
          <div class="muted">
            {{
              effective === 'console' && contextConsole
                ? t('settingsUi.scope.consoleLine', { console: t(`console.${contextConsole}`) })
                : t('settingsUi.scope.globalLine')
            }}
          </div>
        </div>
        <div v-else class="muted">{{ t('settingsUi.appWide') }}</div>
      </template>
      <div v-else class="page-head">
        <h2 class="results-title">{{ t('settingsUi.resultsTitle', { query: query.trim() }) }}</h2>
      </div>

      <section v-for="sec in sections" :key="`${sec.page}.${sec.id}`" class="section">
        <h3 class="overline">
          {{ sec.id === 'results' ? t(msg.page(sec.page)) : t(msg.section(sec.page, sec.id)) }}
        </h3>
        <div class="rows">
          <SettingRowView
            v-for="e in sec.entries"
            :key="e.key"
            :entry="e"
            :row="values[sec.page]?.[e.key]"
            :schema="schema"
            :level="effective"
            :level-name="levelName"
            :scoped="scoped"
            @set="v => set(e.key, v)"
            @reset="reset(e.key)"
            @error="text => emit('error', text)"
            @open="emit('openControls')"
          />
        </div>
      </section>

      <div v-if="query.trim() && sections.length === 0" class="empty">
        {{ t('settingsUi.noResults', { query: query.trim() }) }}
      </div>
      <div v-else-if="onlyChanged && sections.length === 0" class="empty">
        {{ t('settingsUi.emptyOnly') }}
      </div>

      <div v-if="page !== OVERVIEW || query.trim()" class="hint">{{ t('settingsUi.hint') }}</div>
    </main>
  </div>
</template>

<style scoped>
.settings {
  flex: 1;
  min-height: 0;
  display: flex;
}
.cats {
  flex: 0 0 248px;
  padding: 24px 12px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  border-right: 1px solid var(--line);
  overflow-y: auto;
}
.search {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 40px;
  padding: 0 12px;
  background: var(--surface-raised);
  border: 1px solid var(--line-strong);
  border-radius: 8px;
  color: var(--ink-muted);
}
.search input {
  flex: 1;
  min-width: 0;
  background: none;
  border: 0;
  color: var(--ink);
  font: inherit;
  outline: none;
}
.sr {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
}
.group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.group .overline {
  padding: 0 8px 4px;
}
.cat {
  height: 32px;
  padding: 0 8px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}
.cat:hover {
  background: var(--surface-hover);
}
.cat.on {
  background: var(--accent-soft);
  color: var(--accent-text);
}
.cat-name {
  flex: 1;
  text-align: left;
}
.cat.overview {
  font-weight: 600;
}
.count {
  font-size: 12px;
  color: var(--accent-text);
}
.page {
  flex: 1;
  min-width: 0;
  padding: 24px 32px 48px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  overflow-y: auto;
}
.page > * {
  width: 100%;
  max-width: 920px;
}
.page-head {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.page-title {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
  letter-spacing: -0.01em;
}
.muted {
  color: var(--ink-muted);
}
.intro {
  display: flex;
  gap: 10px;
  padding: 12px 14px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
}
.intro :first-child {
  flex: 0 0 auto;
  margin-top: 1px;
  color: var(--accent-text);
}
.scope {
  position: sticky;
  top: 0;
  z-index: 5;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px 16px;
  border-radius: 12px;
}
.scope-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.scope-title {
  font-size: 15px;
  line-height: 20px;
  font-weight: 600;
}
.only {
  display: flex;
  align-items: center;
  gap: 8px;
  border: 0;
  background: none;
  color: var(--ink-muted);
  font-weight: 500;
}
.switch {
  width: 36px;
  height: 20px;
  border-radius: 999px;
  padding: 2px;
  display: flex;
  justify-content: flex-start;
  background: var(--surface-raised);
  border: 1px solid var(--line-strong);
}
.switch.on {
  justify-content: flex-end;
  border: 0;
  background: linear-gradient(var(--accent-crown), var(--accent-deep));
}
.knob {
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: #fff;
}
.levels {
  display: grid;
  gap: 8px;
}
.level {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  min-height: 60px;
  padding: 10px 12px;
  border-radius: var(--radius-10);
  background: var(--surface-raised);
  border: 1px solid var(--line);
  text-align: left;
}
.level.on {
  background: var(--accent-soft);
  border-color: var(--accent-text);
  color: var(--accent-text);
}
.level-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.level-name {
  font-weight: 600;
}
.level-text :deep(.field) {
  min-width: 0;
  height: 26px;
  margin: -3px 0 0 -10px;
  border-color: transparent;
  background: none;
  color: inherit;
  font-weight: 600;
}
.level-sub {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.section .overline {
  margin: 0;
  padding: 0 4px;
}
.section-head {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: baseline;
  gap: 8px;
}
.note {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.results-title {
  margin: 0;
  font-size: 15px;
  line-height: 20px;
  font-weight: 600;
}
/* T01: a row that opens a page (a cards choice, a recent change) */
.link-row {
  display: flex;
  align-items: center;
  gap: 16px;
  min-height: 56px;
  padding: 8px 16px;
  border: 0;
  border-radius: 0;
  background: none;
  border-top: 1px solid var(--line);
  text-align: left;
  color: var(--ink-muted);
}
.link-row:first-child {
  border-top: 0;
}
.link-row:hover {
  background: var(--surface-hover);
}
.link-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.link-name {
  font-weight: 500;
  color: var(--ink);
}
.link-sub {
  font-size: 12px;
  line-height: 16px;
}
.rows {
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
}
.empty {
  padding: 32px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  text-align: center;
  color: var(--ink-muted);
}
.hint {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
</style>
