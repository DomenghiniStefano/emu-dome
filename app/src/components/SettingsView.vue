<script setup lang="ts">
// The Settings section (boards «T01–T20 · Desktop · Impostazioni», ScopeSelector, InheritedRow):
// category list with search and changed counters on the left, the page on the right with the
// scope bar «Stai cambiando le impostazioni di:» fixed at the top while scrolling.
// Every page is built from the Rust schema (`settings_schema`): groups, sections, entries.
//
// Levels: «Tutti i giochi», and «Giochi <console>» for the console of the open game, else of the
// last game played (TODO Cowork, #41: which console when nothing was played). «Solo questo
// gioco» belongs to the game details (L04) and the quick menu (Session), not to this section.
// Controller: the shell's nav scope; L1 / R1 change level (exposed `tab`, called by AppShell).
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Gamepad2, Info, Layers, Search } from '@lucide/vue'
import {
  emu,
  type ConsoleId,
  type SettingEntry,
  type SettingLevel,
  type SettingRow,
  type SettingsSchema,
  type SettingsScope,
  type SettingValue,
} from '../emu'
import { errorText } from '../i18n/labels'
import { library } from '../library'
import { fold, isShown, loadSchema, msg, pageLevels } from '../settingsModel'
import SettingRowView from './SettingRow.vue'

const emit = defineEmits<{ error: [string] }>()
const { t } = useI18n()

const schema = ref<SettingsSchema | null>(null)
const page = ref('video')
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
const contextConsole = ref<ConsoleId | null>(null)
async function findContextConsole() {
  const open = await emu.settings.gameScope().catch(() => null)
  if (open && open.level === 'game') return (contextConsole.value = open.console)
  const last = [...library.games]
    .filter(g => g.lastPlayed !== null)
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))[0]
  contextConsole.value = last?.console ?? null
}

const pageEntries = computed(() => shownOn(page.value))
const levels = computed<SettingLevel[]>(() => {
  const accepted = pageLevels(pageEntries.value)
  const out: SettingLevel[] = ['global']
  if (accepted.has('console') && contextConsole.value) out.push('console')
  return out
})
const scoped = computed(() => levels.value.length > 1)
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
      fold(`${t(msg.name(e.key))} ${t(msg.desc(e.key))} ${e.key}`).includes(q),
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

/** L1 / R1: the previous / next level of the scope bar. */
function tab(dir: -1 | 1) {
  if (!scoped.value) return
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
  unlisten = await emu.settings.onChanged(e => {
    const p = e.key.split('.')[0]
    if (p) void loadPage(p)
  })
})
onBeforeUnmount(() => unlisten?.())

const INTROS = new Set(['bios', 'cores'])
</script>

<template>
  <div v-if="schema" class="settings">
    <nav class="cats" :aria-label="t('settingsUi.categories')">
      <h1 class="cats-title">{{ t('settingsUi.title') }}</h1>
      <label class="search">
        <Search :size="14" :stroke-width="1.75" aria-hidden="true" />
        <span class="sr">{{ t('settingsUi.search') }}</span>
        <input v-model="query" type="search" :placeholder="t('settingsUi.searchPlaceholder')" />
      </label>
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
      <template v-if="!query.trim()">
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
              v-for="l in levels"
              :key="l"
              class="level"
              :class="{ on: l === effective }"
              role="radio"
              :aria-checked="l === effective"
              @click="level = l === 'console' ? 'console' : 'global'"
            >
              <component
                :is="l === 'console' ? Gamepad2 : Layers"
                :size="20"
                :stroke-width="1.75"
                aria-hidden="true"
              />
              <span class="level-text">
                <span class="level-name">{{ levelName(l) }}</span>
                <span class="level-sub">
                  {{
                    l === 'console' && contextConsole
                      ? t('settingsUi.scope.consoleSub', {
                          short: t(`consoleShort.${contextConsole}`),
                        })
                      : t('settingsUi.scope.globalSub')
                  }}
                </span>
              </span>
            </button>
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
        <h2 class="page-title">{{ t('settingsUi.resultsTitle', { query: query.trim() }) }}</h2>
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
          />
        </div>
      </section>

      <div v-if="query.trim() && sections.length === 0" class="empty">
        {{ t('settingsUi.noResults', { query: query.trim() }) }}
      </div>
      <div v-else-if="onlyChanged && sections.length === 0" class="empty">
        {{ t('settingsUi.emptyOnly') }}
      </div>

      <div class="hint">{{ t('settingsUi.hint') }}</div>
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
.cats-title {
  margin: 0;
  padding: 0 8px;
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
  letter-spacing: -0.01em;
}
.search {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  margin: 0 4px;
  padding: 0 10px;
  background: var(--surface-raised);
  border: 1px solid var(--line-strong);
  border-radius: var(--radius-6);
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
