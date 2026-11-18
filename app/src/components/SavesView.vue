<script setup lang="ts">
// The Saves section (board «Salvataggi», V01–V05; «Errori» E03): on the left the games that have
// saves (search, «Importa…»), on the right the chosen game (V02: in-game save, quick saves,
// backups with «Ripristina») or the import (V05). A quick save opens its details (V03).
// No saves at all: E03. The in-game save set aside as unreadable (E04) shows as a note over the
// backups. V06 (two versions) waits for Google Drive sync: ConflictTwoUp.vue is ready for it.
//
// Saves are found from each library game's ROM file name (TODO(game identity, Trello #21)):
// games opened from a file outside the library are not listed.
//
// Controller / keyboard: the shell's nav scope covers this view; the dialogs open their own.
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Save, Search, TriangleAlert } from '@lucide/vue'
import {
  emu,
  type GameSaves,
  type ImportCandidate,
  type LibraryGame,
  type QuickSave,
  type SaveExport,
  type SavesSummary,
  type SettingsScope,
} from '../emu'
import { comboText } from '../controls'
import { errorText, saveWhenLabel, sizeLabel } from '../i18n/labels'
import { library } from '../library'
import FlowDialog from './FlowDialog.vue'
import QuickSaveDialog from './QuickSaveDialog.vue'
import GameCover from './GameCover.vue'
import SlotTile from './SlotTile.vue'
import { savesFocus, savesImportFor } from '../savesFocus'

const emit = defineEmits<{
  openGame: [game: LibraryGame, slot: number]
  /** «Dettagli del gioco» (L04). */
  openDetails: [LibraryGame]
  openLibrary: []
  error: [string]
  notice: [string]
}>()

const { t } = useI18n()

function fail(e: unknown) {
  emit('error', errorText(e))
}

// ---- Settings: `saves.slots` (how many slots), `saves.autoq` (E03's sentence) ----

const GLOBAL: SettingsScope = { level: 'global' }
const slotCount = ref(10)
const autoq = ref('5min')

async function loadSettings() {
  try {
    const rows = await emu.settings.page('saves', GLOBAL)
    const value = (key: string) => rows.find(r => r.key === key)?.value
    slotCount.value = Number(value('saves.slots') ?? 10) || 10
    const q = value('saves.autoq')
    if (typeof q === 'string') autoq.value = q
  } catch (e) {
    fail(e)
  }
}

// ---- V01: the games with saves ----

const summaries = ref<Record<string, SavesSummary>>({})
const loaded = ref(false)

async function loadSummaries() {
  try {
    const list = await emu.saves.summary(
      library.games.map(g => g.path),
      slotCount.value,
    )
    summaries.value = Object.fromEntries(list.map(s => [s.path, s]))
  } catch (e) {
    fail(e)
  }
  loaded.value = library.loaded
}

const hasSaves = (s: SavesSummary | undefined) => !!s && (s.inGame || s.quickCount > 0 || s.broken)
const withSaves = computed(() =>
  library.games
    .filter(g => hasSaves(summaries.value[g.path]))
    .sort(
      (a, b) =>
        (summaries.value[b.path]?.lastChange ?? 0) - (summaries.value[a.path]?.lastChange ?? 0),
    ),
)
const query = ref('')
const shown = computed(() => {
  const q = query.value.trim().toLocaleLowerCase()
  return q ? withSaves.value.filter(g => g.title.toLocaleLowerCase().includes(q)) : withSaves.value
})

function meta(g: LibraryGame): string {
  const s = summaries.value[g.path]
  if (!s) return ''
  const size = sizeLabel(s.bytes)
  return s.quickCount > 0
    ? t('saves.metaQuick', { n: s.quickCount, size }, s.quickCount)
    : t('saves.metaInGame', { size })
}

// ---- V02: one game ----

const mode = ref<'game' | 'import'>('game')
// «Salvataggi» in «Continua a giocare» (L01) opens this section on that game.
const selectedPath = ref<string | null>(savesFocus.value)
savesFocus.value = null
const selected = computed(
  () => withSaves.value.find(g => g.path === selectedPath.value) ?? withSaves.value[0] ?? null,
)
const details = ref<GameSaves | null>(null)

async function loadDetails() {
  const g = selected.value
  if (!g) {
    details.value = null
    return
  }
  try {
    details.value = await emu.saves.game(g.path, slotCount.value)
  } catch (e) {
    fail(e)
  }
}
watch(selected, () => void loadDetails())

async function refresh() {
  await loadSummaries()
  await loadDetails()
}

function pick(g: LibraryGame) {
  selectedPath.value = g.path
  mode.value = 'game'
}

const slotName = (slot: number) => (slot === 0 ? t('qm.auto') : t('qm.slot', { slot }))

async function exportSave(g: LibraryGame, what: SaveExport) {
  try {
    if (await emu.saves.export(g.path, what)) emit('notice', t('saves.exported'))
  } catch (e) {
    fail(e)
  }
}

async function restore(name: string) {
  const g = selected.value
  if (!g) return
  try {
    await emu.saves.restore(g.path, name)
    emit('notice', t('saves.restored'))
  } catch (e) {
    fail(e)
  }
  await refresh()
}

// ---- V03: a quick save's details ----

const quick = ref<QuickSave | null>(null)
let quickReturn: HTMLElement | null = null

function openQuick(q: QuickSave, e: Event) {
  quickReturn = e.currentTarget instanceof HTMLElement ? e.currentTarget : null
  quick.value = q
}
function closeQuick() {
  quick.value = null
  void nextTick(() => quickReturn?.focus())
}
function loadQuick(slot: number) {
  const g = selected.value
  if (!g) return
  quick.value = null
  emit('openGame', g, slot)
}

// ---- V05: import ----

const candidate = ref<ImportCandidate | null>(null)
const target = ref<LibraryGame | null>(null)
/** The game was chosen before the file (L05 «Importa…»): the file name does not change it. */
const targetChosen = ref(false)
const targetFound = ref(false)
const targetSaves = ref<GameSaves | null>(null)
const choosingGame = ref(false)
const importing = ref(false)

const stemOf = (name: string) =>
  (name.split(/[\\/]/).pop() ?? name).replace(/\.[^.]+$/, '').toLocaleLowerCase()

function startImport() {
  mode.value = 'import'
  candidate.value = null
  target.value = null
  targetChosen.value = false
  targetSaves.value = null
}

/** L05 «Importa…»: the import for that game. */
async function startImportFor(path: string) {
  startImport()
  const g = library.games.find(x => x.path === path)
  if (!g) return
  targetChosen.value = true
  await setTarget(g)
}

async function pickFile() {
  try {
    const c = await emu.saves.importPick()
    if (!c) return
    candidate.value = c
    if (targetChosen.value) return
    // The game whose ROM has the same file name, else the same title.
    const stem = stemOf(c.fileName)
    const match =
      library.games.find(g => stemOf(g.path) === stem) ??
      library.games.find(g => g.title.toLocaleLowerCase() === stem) ??
      null
    targetFound.value = match !== null
    if (match) await setTarget(match)
  } catch (e) {
    fail(e)
  }
}

async function setTarget(g: LibraryGame) {
  target.value = g
  try {
    targetSaves.value = await emu.saves.game(g.path, slotCount.value)
  } catch (e) {
    fail(e)
  }
}

async function chooseTarget(g: LibraryGame) {
  choosingGame.value = false
  targetFound.value = false
  targetChosen.value = true
  await setTarget(g)
}

async function doImport() {
  const c = candidate.value
  const g = target.value
  if (!c || !g || importing.value) return
  importing.value = true
  try {
    await emu.saves.import(c.path, g.path)
    emit('notice', t('saves.imported'))
    mode.value = 'game'
    selectedPath.value = g.path
    await refresh()
  } catch (e) {
    fail(e)
  } finally {
    importing.value = false
  }
}

const sortedGames = computed(() =>
  [...library.games].sort((a, b) => a.title.localeCompare(b.title)),
)

// ---- E03 ----

/** The quick-save shortcut as C01 shows it (bindings at «Tutti i giochi», generic pad names). */
const saveCombo = ref('')
async function loadCombo() {
  try {
    const b = await emu.controls.bindings('generic', GLOBAL)
    const row = b.combos.find(c => c.hotkey === 'save')
    saveCombo.value = comboText('generic', b.modifier, row?.pos ?? row?.default ?? null)
  } catch (e) {
    fail(e)
  }
}

const emptyText = computed(() => {
  const combo = saveCombo.value
  return autoq.value === '1min'
    ? t('saves.empty.text1min', { combo })
    : autoq.value === 'never'
      ? t('saves.empty.textNever', { combo })
      : t('saves.empty.text5min', { combo })
})

// ---- Loading and the initial focus ----
// The shell autofocuses on mount, before the saves are known: the focus then sits on the rail's
// current item. Once the list is there, this view's first control takes it (once).

const root = ref<HTMLElement | null>(null)
let focused = false
async function focusInitial() {
  if (focused || !loaded.value) return
  focused = true
  await nextTick()
  const active = document.activeElement
  const onRail = active instanceof HTMLElement && active.getAttribute('aria-current') === 'page'
  if (active && active !== document.body && !onRail) return
  root.value?.querySelector<HTMLElement>('[data-nav-initial]:not(:disabled)')?.focus()
}
watch(loaded, () => void focusInitial())
watch(
  () => library.games,
  () => void refresh(),
)

let unlistenSettings: (() => void) | null = null
onMounted(async () => {
  await loadSettings()
  const importFor = savesImportFor.value
  savesImportFor.value = null
  if (importFor) await startImportFor(importFor)
  await refresh()
  void loadCombo()
  unlistenSettings = await emu.settings.onChanged(c => {
    if (c.key === 'saves.slots' || c.key === 'saves.autoq')
      void loadSettings().then(() => refresh())
    if (c.key === 'pad.combos' || c.key === 'pad.mod') void loadCombo()
  })
})
onUnmounted(() => unlistenSettings?.())
</script>

<template>
  <div ref="root" class="saves">
    <!-- E03: nothing saved yet. -->
    <section v-if="loaded && withSaves.length === 0 && mode === 'game'" class="empty">
      <span class="empty-icon" aria-hidden="true">
        <Save :size="24" :stroke-width="1.75" />
      </span>
      <h1 class="h2">{{ t('saves.empty.title') }}</h1>
      <p class="muted">{{ emptyText }}</p>
      <div class="row">
        <button class="btn tall" data-nav-initial @click="emit('openLibrary')">
          {{ t('saves.empty.openLibrary') }}
        </button>
        <button class="btn tall" @click="startImport">{{ t('saves.empty.import') }}</button>
      </div>
      <p class="faint small">{{ t('saves.empty.formats') }}</p>
    </section>

    <template v-else-if="loaded">
      <!-- V01: the games with saves. -->
      <aside class="list" :aria-label="t('saves.games')">
        <div class="list-head">
          <h1 class="h2">{{ t('shell.saves') }}</h1>
          <button class="btn" @click="startImport">{{ t('saves.import') }}</button>
        </div>
        <label class="search">
          <Search :size="14" :stroke-width="1.75" aria-hidden="true" />
          <span class="sr">{{ t('saves.search') }}</span>
          <input v-model="query" type="search" :placeholder="t('saves.search')" />
        </label>
        <div class="games">
          <button
            v-for="(g, i) in shown"
            :key="g.id"
            class="game"
            :class="{ on: mode === 'game' && g.path === selected?.path }"
            :aria-current="mode === 'game' && g.path === selected?.path ? 'true' : undefined"
            :data-nav-initial="i === 0 ? '' : undefined"
            @click="pick(g)"
          >
            <GameCover class="mini" :game="g" :badge="false" />
            <span class="game-text">
              <span class="game-name">{{ g.title }}</span>
              <span class="muted small num">{{ meta(g) }}</span>
            </span>
            <span v-if="summaries[g.path]?.broken" class="badge warn">
              {{ t('saves.brokenBadge') }}
            </span>
          </button>
          <p v-if="shown.length === 0" class="muted small">{{ t('saves.noneShown') }}</p>
        </div>
      </aside>

      <main class="main">
        <!-- V05: import -->
        <section v-if="mode === 'import'" class="stack">
          <div class="intro">
            <h2 class="h2">{{ t('saves.importer.title') }}</h2>
            <p class="muted">{{ t('saves.importer.intro') }}</p>
          </div>
          <div class="card steps">
            <div class="step">
              <span class="step-n" :class="{ done: candidate }">1</span>
              <div class="step-text">
                <div class="strong">{{ t('saves.importer.file') }}</div>
                <div class="muted small">
                  {{
                    candidate
                      ? t(`saves.importer.format.${candidate.format}`, {
                          file: candidate.fileName,
                        })
                      : t('saves.importer.noFile')
                  }}
                </div>
              </div>
              <button class="btn" data-nav-initial @click="pickFile">
                {{ candidate ? t('saves.importer.change') : t('saves.importer.pick') }}
              </button>
            </div>
            <div class="step">
              <span class="step-n" :class="{ done: target }">2</span>
              <div class="step-text">
                <div class="strong">{{ t('saves.importer.game') }}</div>
                <div class="muted small">
                  {{
                    target
                      ? t(targetFound ? 'saves.importer.gameFound' : 'saves.importer.gameChosen', {
                          title: target.title,
                          console: t(`console.${target.console}`),
                        })
                      : t('saves.importer.gameNone')
                  }}
                </div>
              </div>
              <button class="btn" :disabled="!candidate" @click="choosingGame = true">
                {{ target ? t('saves.importer.otherGame') : t('saves.importer.chooseGame') }}
              </button>
            </div>
            <div v-if="target && targetSaves?.inGame" class="warning small">
              <TriangleAlert class="warn" :size="16" :stroke-width="1.75" aria-hidden="true" />
              <span>
                {{
                  t('saves.importer.replaceWarn', {
                    when: saveWhenLabel(targetSaves.inGame.modified ?? Date.now()),
                  })
                }}
              </span>
            </div>
            <div class="actions">
              <button class="btn" @click="mode = 'game'">{{ t('saves.importer.cancel') }}</button>
              <button
                class="btn btn-primary"
                :disabled="!candidate || !target || importing"
                @click="doImport"
              >
                {{ targetSaves?.inGame ? t('saves.importer.replace') : t('saves.importer.go') }}
              </button>
            </div>
          </div>
          <p class="muted small">{{ t('saves.importer.formats') }}</p>
        </section>

        <!-- V02: one game -->
        <section v-else-if="selected && details" class="stack">
          <div class="game-head">
            <GameCover class="head-cover" :game="selected" :badge="false" />
            <div class="head-text">
              <h2 class="h2">{{ selected.title }}</h2>
              <div class="muted num">
                {{
                  t('saves.gameMeta', {
                    console: t(`console.${selected.console}`),
                    meta: meta(selected),
                  })
                }}
              </div>
            </div>
            <button class="btn" @click="emit('openDetails', selected)">
              {{ t('saves.details') }}
            </button>
          </div>

          <section class="block">
            <h3 class="overline">{{ t('saves.inGameTitle') }}</h3>
            <div class="card row-line">
              <span class="grow num">
                {{
                  details.inGame
                    ? t('saves.inGameLine', {
                        when: saveWhenLabel(details.inGame.modified ?? 0),
                        size: sizeLabel(details.inGame.size),
                      })
                    : t('saves.noInGame')
                }}
              </span>
              <button
                v-if="details.inGame"
                class="btn"
                @click="exportSave(selected, { kind: 'inGame' })"
              >
                {{ t('saves.export') }}
              </button>
            </div>
          </section>

          <section class="block">
            <h3 class="overline">{{ t('saves.quickTitle') }}</h3>
            <!-- Each tile: «Gioca da qui» opens the game on that slot, «…» the other actions (V03).
                 The date only: a quick save has no play time of its own (Cowork, #37).
                 `outdated`: an older version of its core made it (saves.rs, `is_outdated`). -->
            <div v-if="details.quick.length" class="slots">
              <SlotTile
                v-for="q in details.quick"
                :key="q.slot"
                :name="slotName(q.slot)"
                :when="q.savedAt !== null ? saveWhenLabel(q.savedAt) : null"
                :thumb-url="q.thumbUrl"
                :outdated="q.outdated"
                @play="emit('openGame', selected, q.slot)"
                @more="e => openQuick(q, e)"
              />
            </div>
            <p v-else class="muted">{{ t('saves.noQuick') }}</p>
          </section>

          <section class="block">
            <h3 class="overline">{{ t('saves.backupsTitle') }}</h3>
            <!-- E04: the unreadable in-game save was set aside. -->
            <div v-if="details.broken.length" class="warning small">
              <TriangleAlert class="warn" :size="16" :stroke-width="1.75" aria-hidden="true" />
              <span>{{ t('saves.brokenNote', { file: details.broken[0] }) }}</span>
            </div>
            <div v-if="details.backups.length" class="card rows">
              <div v-for="b in details.backups" :key="b.name" class="row-line">
                <span class="grow num">{{ saveWhenLabel(b.at) }}</span>
                <span class="muted small">{{ t(`saves.reason.${b.reason}`) }}</span>
                <button class="btn" @click="restore(b.name)">{{ t('saves.restore') }}</button>
              </div>
            </div>
            <p v-else class="muted">{{ t('saves.noBackups') }}</p>
            <p class="muted small">{{ t('saves.backupNote') }}</p>
          </section>
        </section>
      </main>
    </template>

    <!-- V03: a quick save -->
    <QuickSaveDialog
      v-if="quick && selected"
      :game="selected"
      :quick="quick"
      @close="closeQuick"
      @play="loadQuick"
      @changed="refresh"
      @error="text => emit('error', text)"
      @notice="text => emit('notice', text)"
    />

    <!-- V05: «Scegli un altro gioco…» -->
    <FlowDialog
      v-if="choosingGame"
      :title="t('saves.importer.pickTitle')"
      @back="choosingGame = false"
    >
      <div class="pick-list">
        <button
          v-for="(g, i) in sortedGames"
          :key="g.id"
          class="game"
          :class="{ on: g.id === target?.id }"
          :data-nav-initial="(target ? g.id === target.id : i === 0) ? '' : undefined"
          @click="chooseTarget(g)"
        >
          <GameCover class="mini" :game="g" :badge="false" />
          <span class="game-text">
            <span class="game-name">{{ g.title }}</span>
            <span class="muted small">{{ t(`console.${g.console}`) }}</span>
          </span>
        </button>
      </div>
      <template #actions>
        <button class="btn" @click="choosingGame = false">{{ t('saves.importer.cancel') }}</button>
      </template>
    </FlowDialog>
  </div>
</template>

<style scoped>
.saves {
  flex: 1;
  min-height: 0;
  display: flex;
}
.h2 {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
  letter-spacing: -0.01em;
}
.muted {
  margin: 0;
  color: var(--ink-muted);
}
.faint {
  margin: 0;
  color: var(--ink-faint);
}
.small {
  font-size: 12px;
  line-height: 16px;
}
.strong {
  font-weight: 500;
}
.p {
  margin: 0;
}
.grow {
  flex: 1;
  min-width: 0;
}
.sr {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
}
.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.btn.tall {
  height: 36px;
}
.warn {
  flex: 0 0 auto;
  color: var(--warning);
}
.card {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
}

/* E03 */
.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 32px;
  text-align: center;
}
.empty .muted {
  max-width: 460px;
}
.empty-icon {
  width: 56px;
  height: 56px;
  border-radius: 16px;
  background: var(--surface-raised);
  border: 1px solid var(--line);
  display: grid;
  place-items: center;
  color: var(--ink-muted);
}
.row {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 8px;
  margin-top: 4px;
}

/* V01 */
.list {
  flex: 0 0 360px;
  min-height: 0;
  overflow: auto;
  padding: 24px 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  border-right: 1px solid var(--line);
}
.list-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.search {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  padding: 0 10px;
  border-radius: var(--radius-6);
  background: var(--surface-raised);
  border: 1px solid var(--line-strong);
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
.games,
.pick-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.pick-list {
  max-height: 50vh;
  overflow: auto;
}
.game {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  border: 0;
  border-radius: 8px;
  background: none;
  text-align: left;
}
.game:hover {
  background: var(--surface-hover);
}
.game.on {
  background: var(--accent-soft);
}
.mini {
  flex: 0 0 auto;
  width: 28px;
  padding: 0;
  border-radius: 4px;
  font-size: 0;
}
.game-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.game-name {
  font-weight: 500;
  overflow-wrap: anywhere;
}
.badge {
  font-size: 11px;
  line-height: 16px;
  padding: 0 6px;
  border-radius: 4px;
  white-space: nowrap;
}
.badge.warn {
  background: color-mix(in srgb, var(--warning) 14%, transparent);
}

/* Right column */
.main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: 24px 32px 48px;
}
.stack {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.intro {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.game-head {
  display: flex;
  align-items: center;
  gap: 14px;
}
.head-cover {
  flex: 0 0 auto;
  width: 48px;
  padding: 0;
  border-radius: var(--radius-6);
  font-size: 0;
}
.head-text {
  flex: 1;
  min-width: 0;
}
.block {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.overline {
  margin: 0;
}
.row-line {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
}
.rows {
  overflow: hidden;
}
.rows .row-line + .row-line {
  border-top: 1px solid var(--line);
}
.slots {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}
/* V05 */
.steps {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.step {
  display: flex;
  align-items: center;
  gap: 12px;
}
.step-text {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}
.step-n {
  flex: 0 0 auto;
  width: 24px;
  height: 24px;
  border-radius: 999px;
  display: grid;
  place-items: center;
  font-weight: 600;
  font-size: 12px;
  background: var(--surface-raised);
  color: var(--ink-muted);
}
.step-n.done {
  background: color-mix(in srgb, var(--success) 14%, transparent);
  color: var(--success);
}
.warning {
  display: flex;
  gap: 10px;
  padding: 10px 12px;
  border-radius: var(--radius-10);
  background: color-mix(in srgb, var(--warning) 14%, transparent);
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
