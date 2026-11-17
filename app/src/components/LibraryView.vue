<script setup lang="ts">
// The Library section (board L01 · Desktop · Libreria): the «Riprendi» hero for the last game
// played, the console chips, the order (`library.sort`, Global) and the L03 panel, the cover
// grid. Around it: E01 banners for folders that are not reachable (their games stay, faded),
// the scan progress, L11 «N giochi aggiunti» after a scan, L12 to choose a cover.
// No games at all: L02 (LibraryEmpty).
//
// Controller / keyboard: the shell's nav scope covers this view; L03 and L12 open their own.
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronDown, FolderX, ImageIcon, Play, Plus, TriangleAlert, X } from '@lucide/vue'
import { emu, type ConsoleId, type LibraryGame, type SettingsScope } from '../emu'
import { errorText, lastPlayedLabel, playTimeLabel } from '../i18n/labels'
import { chooseFolder, library, replaceFolder } from '../library'
import {
  CONSOLES,
  DEFAULT_SORT,
  isSort,
  SHOW_KEYS,
  shown,
  sorted,
  type ShowKey,
  type Sort,
} from '../librarySort'
import CoverPicker from './CoverPicker.vue'
import GameCover from './GameCover.vue'
import LibraryEmpty from './LibraryEmpty.vue'
import LibraryFilters from './LibraryFilters.vue'

defineProps<{ error: string | null }>()
const emit = defineEmits<{ open: [LibraryGame]; openFile: []; error: [string] }>()

const { t } = useI18n()

// ---- Order: the `library.sort` setting, Global level ----

const GLOBAL: SettingsScope = { level: 'global' }
const sort = ref<Sort>(DEFAULT_SORT)

async function loadSort() {
  try {
    const row = (await emu.settings.page('library', GLOBAL)).find(r => r.key === 'library.sort')
    sort.value = isSort(row?.value) ? row.value : DEFAULT_SORT
  } catch (e) {
    emit('error', errorText(e))
  }
}

async function setSort(s: Sort) {
  sort.value = s
  try {
    await emu.settings.set('library.sort', GLOBAL, s)
  } catch (e) {
    emit('error', errorText(e))
    await loadSort()
  }
}

// ---- Filters: L03 «Mostra» and the console chips ----

const show = reactive<Record<ShowKey, boolean>>({
  favourites: false,
  unplayed: false,
  here: false,
  hidden: false,
})
const consoleFilter = ref<ConsoleId | 'all'>('all')
const filtersOpen = ref(false)
const sortButton = ref<HTMLButtonElement | null>(null)

function closeFilters() {
  filtersOpen.value = false
  void nextTick(() => sortButton.value?.focus())
}

async function resetFilters() {
  for (const k of SHOW_KEYS) show[k] = false
  try {
    await emu.settings.reset('library.sort', GLOBAL)
  } catch (e) {
    emit('error', errorText(e))
  }
  await loadSort()
}

const base = computed(() => shown(library.games, show))
const chips = computed(() => {
  const present = CONSOLES.filter(c => base.value.some(g => g.console === c))
  return [
    { id: 'all' as const, count: base.value.length },
    ...present.map(c => ({ id: c, count: base.value.filter(g => g.console === c).length })),
  ]
})
// A chip whose console left the list (filters, removal) falls back to «Tutti».
watch(chips, list => {
  if (!list.some(c => c.id === consoleFilter.value)) consoleFilter.value = 'all'
})
const games = computed(() =>
  sorted(
    consoleFilter.value === 'all'
      ? base.value
      : base.value.filter(g => g.console === consoleFilter.value),
    sort.value,
  ),
)

// ---- The «Riprendi» hero: the last game played, whatever the filters ----

const hero = computed(() =>
  library.games
    .filter(g => !g.hidden && g.lastPlayed !== null)
    .reduce<LibraryGame | null>(
      (best, g) => (best === null || (g.lastPlayed ?? 0) > (best.lastPlayed ?? 0) ? g : best),
      null,
    ),
)

const consoleName = (c: ConsoleId) => t(`console.${c}`)

function meta(g: LibraryGame): string {
  if (g.lastPlayed === null) return t('library.neverPlayed')
  return t('library.meta', {
    time: playTimeLabel(g.playTimeSecs),
    when: lastPlayedLabel(g.lastPlayed),
  })
}

function open(g: LibraryGame) {
  if (!g.available) {
    emit('error', errorText({ code: 'libraryGameUnavailable', params: { id: g.id } }))
    return
  }
  emit('open', g)
}

// ---- Folders: L02 / «Aggiungi giochi», E01 ----

async function addFolder() {
  const failed = await chooseFolder()
  if (failed) emit('error', failed)
}

async function replace(path: string) {
  const failed = await replaceFolder(path)
  if (failed) emit('error', failed)
}

const unreachable = computed(() =>
  library.folders
    .filter(f => !f.reachable)
    .map(f => ({ path: f.path, count: library.games.filter(g => g.folder === f.path).length })),
)

// ---- L11: what the last scan added ----

const summaryCards = computed(() => {
  const per = library.summary?.perConsole ?? {}
  const cards: Array<{ key: string; count: number }> = [
    { key: 'console.gba', count: per.gba ?? 0 },
    { key: 'console.nds', count: per.nds ?? 0 },
    { key: 'console.3ds', count: per['3ds'] ?? 0 },
    { key: 'library.added.gbAndColor', count: (per.gb ?? 0) + (per.gbc ?? 0) },
  ]
  return cards.filter(c => c.count > 0)
})

// ---- L12 ----

const coverFor = ref<LibraryGame | null>(null)
let coverReturn: HTMLElement | null = null

function pickCover(g: LibraryGame, e: Event) {
  coverReturn = e.currentTarget instanceof HTMLElement ? e.currentTarget : null
  coverFor.value = g
}
function closeCover() {
  coverFor.value = null
  void nextTick(() => coverReturn?.focus())
}

// ---- Initial focus ----
// The shell autofocuses on mount, before the library has loaded: then the focus sits on the rail's
// current item. Once the library is there, its own first control takes it (once).

const root = ref<HTMLElement | null>(null)
let focused = false
async function focusInitial() {
  if (focused || !library.loaded) return
  focused = true
  await nextTick()
  const active = document.activeElement
  const onRail = active instanceof HTMLElement && active.getAttribute('aria-current') === 'page'
  if (active && active !== document.body && !onRail) return
  root.value?.querySelector<HTMLElement>('[data-nav-initial]:not(:disabled)')?.focus()
}
watch(() => library.loaded, focusInitial)

let unlistenSettings: (() => void) | null = null
onMounted(async () => {
  void focusInitial()
  await loadSort()
  unlistenSettings = await emu.settings.onChanged(c => {
    if (c.key === 'library.sort') void loadSort()
  })
})
onUnmounted(() => unlistenSettings?.())
</script>

<template>
  <div ref="root" class="library">
    <!-- E01: a library folder is not reachable; its games stay listed, faded. -->
    <div v-for="f in unreachable" :key="f.path" class="banner" role="status">
      <FolderX class="warn" :size="18" :stroke-width="1.75" aria-hidden="true" />
      <span class="banner-text">{{
        t('library.unreachable.text', { path: f.path, count: f.count }, f.count)
      }}</span>
      <button class="btn" @click="replace(f.path)">{{ t('library.unreachable.choose') }}</button>
    </div>

    <!-- Scan in progress (S03's bar, indeterminate: the total is not known up front). -->
    <div v-if="library.scanning" class="scan" role="status">
      <span class="scan-title">{{ t('library.scan.title') }}</span>
      <span class="bar" aria-hidden="true"><span></span></span>
      <span v-if="library.progress" class="muted num">
        {{ t('library.scan.found', { found: library.progress.found }, library.progress.found) }}
      </span>
    </div>

    <!-- L11: what the last scan added. -->
    <section
      v-if="library.summary"
      class="added glass"
      :aria-label="t('library.added.title', { n: library.summary.added }, library.summary.added)"
    >
      <div class="added-head">
        <h2 class="h2">
          {{ t('library.added.title', { n: library.summary.added }, library.summary.added) }}
        </h2>
        <button
          class="icon-btn"
          :aria-label="t('library.added.close')"
          :title="t('library.added.close')"
          @click="library.summary = null"
        >
          <X :size="16" :stroke-width="1.75" aria-hidden="true" />
        </button>
      </div>
      <div v-if="summaryCards.length" class="added-cards">
        <div v-for="c in summaryCards" :key="c.key" class="added-card">
          <div class="added-count num">{{ c.count }}</div>
          <div class="muted">{{ t(c.key) }}</div>
        </div>
      </div>
    </section>

    <LibraryEmpty
      v-if="library.loaded && library.games.length === 0"
      :error="error ?? library.error"
      @choose-folder="addFolder"
      @open-file="emit('openFile')"
    />

    <template v-else-if="library.loaded">
      <!-- «Riprendi»: the last game played. -->
      <section v-if="hero" class="hero glass" :aria-label="t('library.hero.label')">
        <GameCover
          class="hero-cover"
          :game="hero"
          :badge="false"
          :style="{ boxShadow: `0 0 28px ${hero.colour}80` }"
        />
        <div class="hero-body">
          <div class="overline">{{ t('library.hero.label') }}</div>
          <h1 class="hero-title">{{ hero.title }}</h1>
          <div class="hero-meta num">
            {{
              t('library.hero.meta', {
                console: consoleName(hero.console),
                time: playTimeLabel(hero.playTimeSecs, true),
                when: lastPlayedLabel(hero.lastPlayed ?? 0),
              })
            }}
          </div>
          <div class="hero-actions">
            <button
              class="btn btn-primary"
              data-nav-initial
              :disabled="!hero.available"
              @click="open(hero)"
            >
              <Play :size="16" fill="currentColor" :stroke-width="0" aria-hidden="true" />
              {{ t('library.hero.resume') }}
            </button>
            <button class="btn tall" disabled :title="t('placeholder.line')">
              {{ t('library.hero.chooseSave') }}
            </button>
            <button class="btn tall" disabled :title="t('placeholder.line')">
              {{ t('library.hero.details') }}
            </button>
          </div>
        </div>
        <figure v-if="hero.lastStateThumbUrl" class="hero-shot">
          <img :src="hero.lastStateThumbUrl" alt="" />
          <figcaption>{{ t('library.hero.autoSave') }}</figcaption>
        </figure>
      </section>

      <div class="toolbar">
        <h2 class="h2">{{ t('library.yourGames') }}</h2>
        <div class="chips" role="group" :aria-label="t('library.filterConsole')">
          <button
            v-for="c in chips"
            :key="c.id"
            class="chip"
            :class="{ on: c.id === consoleFilter }"
            :aria-pressed="c.id === consoleFilter"
            :data-nav-initial="!hero && c.id === 'all' ? '' : undefined"
            @click="consoleFilter = c.id"
          >
            {{ c.id === 'all' ? t('library.all') : consoleName(c.id) }}
            <span class="count num">{{ c.count }}</span>
          </button>
        </div>
        <div class="spacer"></div>
        <div class="sort-wrap">
          <button
            ref="sortButton"
            class="btn muted-btn"
            :aria-expanded="filtersOpen"
            aria-haspopup="dialog"
            @click="filtersOpen = !filtersOpen"
          >
            {{ t(`library.sortButton.${sort}`) }}
            <ChevronDown :size="14" :stroke-width="1.75" aria-hidden="true" />
          </button>
          <LibraryFilters
            v-if="filtersOpen"
            :sort="sort"
            :show="show"
            @sort="setSort"
            @toggle="(k: ShowKey) => (show[k] = !show[k])"
            @reset="resetFilters"
            @close="closeFilters"
          />
        </div>
        <button class="btn" @click="addFolder">
          <Plus :size="16" :stroke-width="1.75" aria-hidden="true" />
          {{ t('library.addGames') }}
        </button>
      </div>

      <div class="grid">
        <div v-for="g in games" :key="g.id" class="card" :class="{ off: !g.available }">
          <button
            class="cover-btn"
            :aria-label="t('library.open', { title: g.title })"
            :aria-disabled="!g.available || undefined"
            @click="open(g)"
          >
            <GameCover :game="g" />
          </button>
          <div class="info">
            <div class="name-row">
              <div class="name">{{ g.title }}</div>
              <button
                class="icon-btn pick"
                :aria-label="t('library.cover.choose', { title: g.title })"
                :title="t('library.cover.choose', { title: g.title })"
                @click="pickCover(g, $event)"
              >
                <ImageIcon :size="14" :stroke-width="1.75" aria-hidden="true" />
              </button>
            </div>
            <div class="muted small num">{{ meta(g) }}</div>
          </div>
        </div>
      </div>
      <p v-if="games.length === 0" class="muted">{{ t('library.noneShown') }}</p>

      <p v-if="unreachable.length" class="muted">
        <TriangleAlert class="warn inline" :size="14" :stroke-width="1.75" aria-hidden="true" />
        {{ t('library.unreachable.hint') }}
      </p>
    </template>

    <CoverPicker v-if="coverFor" :game="coverFor" @close="closeCover" />
  </div>
</template>

<style scoped>
.library {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 24px;
}
.glass {
  background: var(--glass);
  border: 1px solid var(--line);
  border-radius: var(--radius-16);
  box-shadow: inset 0 1px 0 var(--glass-edge);
  backdrop-filter: blur(24px) saturate(140%);
}
.muted {
  margin: 0;
  color: var(--ink-muted);
}
.small {
  font-size: 12px;
  line-height: 16px;
}
.h2 {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
  letter-spacing: -0.01em;
}
.warn {
  flex: 0 0 auto;
  color: var(--warning);
}
.warn.inline {
  vertical-align: -2px;
}
.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.icon-btn {
  width: 32px;
  height: 32px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  color: var(--ink-muted);
  display: grid;
  place-items: center;
}
.icon-btn:hover {
  background: var(--surface-hover);
}

/* E01 */
.banner {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-radius: var(--radius-10);
  background: color-mix(in srgb, var(--warning) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--warning) 33%, transparent);
}
.banner-text {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}
.banner .btn {
  white-space: nowrap;
}

/* Scan */
.scan {
  display: flex;
  align-items: center;
  gap: 12px;
}
.scan-title {
  font-weight: 500;
  white-space: nowrap;
}
.bar {
  flex: 1;
  max-width: 320px;
  height: 6px;
  border-radius: 999px;
  background: var(--surface-raised);
  overflow: hidden;
}
.bar span {
  display: block;
  width: 30%;
  height: 100%;
  border-radius: inherit;
  background: var(--accent-text);
  animation: sweep 1.4s var(--ease-out) infinite;
}
@keyframes sweep {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(340%);
  }
}
@media (prefers-reduced-motion: reduce) {
  .bar span {
    animation: none;
    width: 100%;
    opacity: 0.5;
  }
}

/* L11 */
.added {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.added-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.added-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 12px;
}
.added-card {
  padding: 14px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
}
.added-count {
  font-size: 32px;
  line-height: 38px;
  font-weight: 600;
}

/* Hero */
.hero {
  padding: 20px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 24px;
}
.hero-cover {
  width: 120px;
  font-size: 14px;
  line-height: 17px;
  padding: 10px;
}
.hero-body {
  flex: 1 1 320px;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.hero-title {
  margin: 0;
  font-size: 32px;
  line-height: 38px;
  font-weight: 600;
  letter-spacing: -0.02em;
}
.hero-meta {
  color: var(--ink-muted);
  font-size: 15px;
  line-height: 22px;
}
.hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}
.btn.tall {
  height: 40px;
  padding: 0 14px;
}
.hero-shot {
  margin: 0;
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.hero-shot img {
  display: block;
  width: 160px;
  padding: 6px;
  background: #000;
  border: 1px solid var(--line);
  border-radius: var(--radius-10);
  image-rendering: pixelated;
}
.hero-shot figcaption {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}

/* Toolbar */
.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}
.toolbar .h2 {
  margin-right: 8px;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.chip {
  height: 32px;
  padding: 0 12px;
  border-radius: var(--radius-6);
  border: 1px solid var(--line);
  background: var(--surface-raised);
  font-weight: 500;
}
.chip:hover {
  background: var(--surface-hover);
}
.chip.on {
  background: var(--accent-soft);
  color: var(--accent-text);
  border-color: color-mix(in srgb, var(--violet) 40%, transparent);
}
.count {
  opacity: 0.8;
}
.spacer {
  flex: 1;
}
.sort-wrap {
  position: relative;
}
.muted-btn {
  color: var(--ink-muted);
  font-weight: 400;
}

/* Grid */
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(156px, 1fr));
  gap: 16px;
}
.card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
.cover-btn {
  display: block;
  padding: 0;
  border: 0;
  border-radius: var(--radius-10);
  background: none;
  box-shadow: 0 1px 2px #00000052;
  transition:
    transform 150ms var(--ease-out),
    box-shadow 150ms;
}
/* Lift on hover and on focus (board L01). */
.cover-btn:hover,
.cover-btn:focus-visible,
:root[data-nav] .cover-btn:focus,
.cover-btn[data-nav-focus] {
  transform: translateY(-2px);
  box-shadow: 0 10px 24px #00000073;
}
:root[data-nav] .cover-btn:focus,
:root[data-nav] .cover-btn[data-nav-focus] {
  box-shadow:
    0 10px 24px #00000073,
    0 0 0 4px var(--accent-soft);
}
/* E01: not reachable, shown faded and not launchable. */
/* (On the cover, not the button: the button keeps the focus ring.) */
.card.off .cover-btn {
  cursor: not-allowed;
}
.card.off .cover {
  opacity: 0.45;
  outline: 1px dashed var(--line-strong);
  outline-offset: -1px;
}
.card.off .info {
  opacity: 0.6;
}
.info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.name-row {
  display: flex;
  align-items: flex-start;
  gap: 4px;
}
.name {
  flex: 1;
  min-width: 0;
  font-weight: 500;
  overflow-wrap: anywhere;
}
/* «Scegli la copertina» shows with the pointer over the card or the focus in it. */
.pick {
  flex: 0 0 auto;
  width: 24px;
  height: 24px;
  margin: -3px -4px 0 0;
  opacity: 0;
  transition: opacity 150ms;
}
.card:hover .pick,
.card:focus-within .pick {
  opacity: 1;
}
</style>
