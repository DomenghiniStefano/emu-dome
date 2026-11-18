<script setup lang="ts">
// The Library section (board L01 · Desktop · Libreria, Libreria2 17/11): «Continua a giocare»
// with up to four games played last («Riprendi» opens one and loads its newest quick save,
// «Salvataggi» shows its saves), then «I tuoi giochi»: the console chips, the order
// (`library.sort`, Global) and the L03 panel, the cover grid. Around it: E01 banners for folders that are not reachable (their games stay, faded),
// the scan progress, L11 «N giochi aggiunti» after a scan.
// A cover (#59 item 2): a click opens its game details (L04, #34), as do a right click and X on
// the controller; with the pointer over it or the focus in it, a ring, «Gioca» (starts the game)
// and «…» (CoverMenu: favourite, hide; the cover itself is chosen from L04 only, Cowork #40).
// Back from the details focuses the cover again (`detailsReturn`).
// No games at all: L02 (LibraryEmpty).
//
// Controller / keyboard: the shell's nav scope covers this view; L03 opens its own.
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronDown, Ellipsis, FolderX, Play, Plus, TriangleAlert, X } from '@lucide/vue'
import { AUTO_SLOT, emu, type ConsoleId, type LibraryGame, type SettingsScope } from '../emu'
import { detailsReturn, resumePoint, type ResumePoint } from '../gameDetails'
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
import CoverMenu from './CoverMenu.vue'
import GameCover from './GameCover.vue'
import LibraryEmpty from './LibraryEmpty.vue'
import LibraryFilters from './LibraryFilters.vue'

defineProps<{ error: string | null }>()
const emit = defineEmits<{
  /** `slot`: the quick save to load once the game runs («Riprendi»). */
  open: [game: LibraryGame, slot?: number]
  /** «Salvataggi» on a «Continua a giocare» card: the Saves section on that game. */
  openSaves: [LibraryGame]
  /** L04: the game's details. */
  details: [LibraryGame]
  openFile: []
  error: [string]
}>()

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

/** «Togli i filtri» under «Nessun gioco corrisponde…»: the L03 switches and the console chip. */
const filtering = computed(() => consoleFilter.value !== 'all' || SHOW_KEYS.some(k => show[k]))
function clearFilters() {
  for (const k of SHOW_KEYS) show[k] = false
  consoleFilter.value = 'all'
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

// ---- «Continua a giocare»: the four games played last, whatever the filters ----

const CONTINUE = 4
const recent = computed(() =>
  library.games
    .filter(g => !g.hidden && g.lastPlayed !== null)
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))
    .slice(0, CONTINUE),
)

/** Where «Riprendi» picks up: the newest quick save (auto or a slot), else the in-game save. */
const resume = ref<Record<string, ResumePoint>>({})

async function loadResume() {
  const games = recent.value
  if (games.length === 0) return
  let slots = 10
  try {
    const row = (await emu.settings.page('saves', GLOBAL)).find(r => r.key === 'saves.slots')
    slots = Number(row?.value ?? 10) || 10
  } catch (e) {
    emit('error', errorText(e))
  }
  const points = await Promise.all(
    games.map(async (g): Promise<[string, ResumePoint]> => {
      try {
        return [g.id, resumePoint(await emu.saves.game(g.path, slots))]
      } catch {
        // Saves unreadable for now: open the game as «Gioca» would.
        return [g.id, { slot: null, thumbUrl: null, none: true }]
      }
    }),
  )
  resume.value = Object.fromEntries(points)
}
watch(
  () => recent.value.map(g => `${g.id}:${g.lastPlayed}`).join(),
  () => void loadResume(),
  { immediate: true },
)

function resumeFrom(p: ResumePoint | undefined): string {
  if (!p || p.none) return ''
  if (p.slot === null) return t('library.continue.fromInGame')
  if (p.slot === AUTO_SLOT) return t('library.continue.fromAuto')
  return t('library.continue.fromSlot', { slot: p.slot })
}

function resumeGame(g: LibraryGame) {
  if (!g.available) {
    emit('error', errorText({ code: 'libraryGameUnavailable', params: { id: g.id } }))
    return
  }
  emit('open', g, resume.value[g.id]?.slot ?? undefined)
}

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

// ---- «…» on a cover ----

const menuFor = ref<string | null>(null)
let menuButton: HTMLElement | null = null

function toggleMenu(g: LibraryGame, e: Event) {
  menuButton = e.currentTarget instanceof HTMLElement ? e.currentTarget : null
  menuFor.value = menuFor.value === g.id ? null : g.id
}
function closeMenu() {
  menuFor.value = null
  void nextTick(() => menuButton?.focus())
}
async function menuFlags(g: LibraryGame, flags: { favourite?: boolean; hidden?: boolean }) {
  closeMenu()
  try {
    await emu.library.setFlags(g.id, flags)
  } catch (e) {
    emit('error', errorText(e))
  }
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
  // Back from a game's details: its cover.
  const back = detailsReturn.value
  detailsReturn.value = null
  const cover = back
    ? root.value?.querySelector<HTMLElement>(`[data-game="${CSS.escape(back)}"] .cover-btn`)
    : null
  if (cover) {
    cover.focus({ preventScroll: true })
    cover.scrollIntoView({ block: 'nearest' })
    return
  }
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
      <!-- «Continua a giocare»: the games played last. -->
      <section v-if="recent.length" class="continue" :aria-label="t('library.continue.title')">
        <div class="continue-head">
          <h1 class="h2">{{ t('library.continue.title') }}</h1>
          <span class="muted small">{{ t('library.continue.lead') }}</span>
        </div>
        <div class="continue-row">
          <div
            v-for="(g, i) in recent"
            :key="g.id"
            class="continue-card"
            :class="{ off: !g.available }"
          >
            <span class="shot" aria-hidden="true" :style="{ background: g.colour }">
              <img v-if="resume[g.id]?.thumbUrl" :src="resume[g.id]?.thumbUrl ?? ''" alt="" />
            </span>
            <div class="continue-body">
              <div class="continue-title">{{ g.title }}</div>
              <div class="muted small num">
                {{
                  t('library.continue.meta', {
                    console: t(`consoleShort.${g.console}`),
                    time: playTimeLabel(g.playTimeSecs),
                    when: lastPlayedLabel(g.lastPlayed ?? 0),
                  })
                }}
              </div>
              <div class="continue-actions">
                <button
                  class="btn small-btn"
                  :class="{ 'btn-primary': i === 0 }"
                  :data-nav-initial="i === 0 ? '' : undefined"
                  :disabled="!g.available"
                  @click="resumeGame(g)"
                >
                  {{ t(resume[g.id]?.none ? 'library.hero.play' : 'library.hero.resume') }}
                </button>
                <button class="btn small-btn" @click="emit('openSaves', g)">
                  {{ t('shell.saves') }}
                </button>
              </div>
              <div v-if="resumeFrom(resume[g.id])" class="from">
                {{ resumeFrom(resume[g.id]) }}
              </div>
            </div>
          </div>
        </div>
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
            :data-nav-initial="!recent.length && c.id === 'all' ? '' : undefined"
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
        <div
          v-for="g in games"
          :key="g.id"
          class="card"
          :class="{ off: !g.available }"
          :data-game="g.id"
        >
          <div
            class="frame"
            :class="{ open: menuFor === g.id }"
            @contextmenu.prevent="emit('details', g)"
          >
            <GameCover :game="g" />
            <!-- The cover's click area stops above the action strip, so the controller reaches
                 «Gioca» and «…» by moving down from it (nav.ts moves by rects). -->
            <button
              class="cover-btn"
              :aria-label="t('library.openDetails', { title: g.title })"
              @click="emit('details', g)"
            ></button>
            <div class="cover-actions">
              <button class="play" :aria-disabled="!g.available || undefined" @click="open(g)">
                <Play :size="14" fill="currentColor" :stroke-width="0" aria-hidden="true" />
                {{ t('library.hero.play') }}
              </button>
              <div class="more-wrap">
                <button
                  class="more"
                  :aria-label="t('library.moreFor', { title: g.title })"
                  :title="t('library.more')"
                  aria-haspopup="menu"
                  :aria-expanded="menuFor === g.id"
                  @click="toggleMenu(g, $event)"
                >
                  <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
                </button>
                <CoverMenu
                  v-if="menuFor === g.id"
                  :game="g"
                  :items="['favourite', 'hide']"
                  @flags="f => menuFlags(g, f)"
                  @close="closeMenu"
                />
              </div>
            </div>
          </div>
          <div class="info">
            <div class="name">{{ g.title }}</div>
            <div class="muted small num">{{ meta(g) }}</div>
          </div>
        </div>
      </div>
      <div v-if="games.length === 0" class="none-shown">
        <p class="muted">{{ t('library.noneShown') }}</p>
        <button v-if="filtering" class="btn" @click="clearFilters">
          {{ t('library.clearFilters') }}
        </button>
      </div>

      <p v-if="unreachable.length" class="muted">
        <TriangleAlert class="warn inline" :size="14" :stroke-width="1.75" aria-hidden="true" />
        {{ t('library.unreachable.hint') }}
      </p>
    </template>
  </div>
</template>

<style scoped>
.none-shown {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}
.none-shown p {
  margin: 0;
}
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

/* «Continua a giocare» */
.continue {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.continue-head {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: baseline;
  gap: 8px;
}
.continue-row {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 12px;
}
.continue-card {
  display: flex;
  gap: 12px;
  padding: 10px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  min-width: 0;
}
.continue-card.off .shot {
  opacity: 0.45;
}
.shot {
  flex: 0 0 auto;
  width: 96px;
  aspect-ratio: 3 / 2;
  border-radius: var(--radius-6);
  overflow: hidden;
}
.shot img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  image-rendering: pixelated;
}
.continue-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.continue-title {
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.continue-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: auto;
}
.small-btn {
  height: 28px;
}
.from {
  font-size: 11px;
  line-height: 14px;
  color: var(--ink-faint);
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
.frame {
  position: relative;
  border-radius: var(--radius-10);
  box-shadow: 0 1px 2px #00000052;
  outline: 2px solid transparent;
  outline-offset: 2px;
  transition: outline-color 150ms;
}
/* The cover's click area; it stops above the action strip (see the template). */
.cover-btn {
  position: absolute;
  inset: 0 0 52px;
  padding: 0;
  border: 0;
  border-radius: var(--radius-10) var(--radius-10) 0 0;
  background: none;
}
/* Pointer over the cover or focus in it (board L01, second cover): the ring, then «Gioca» and «…».
   The ring is the frame's, so the buttons inside draw none of their own. */
.frame:hover,
.frame:focus-within,
.frame.open {
  outline-color: var(--focus-ring);
}
:root[data-nav] .frame :focus,
:root[data-nav] .frame [data-nav-focus] {
  outline: none;
}
:root[data-nav] .frame .cover-actions button:focus,
:root[data-nav] .frame .cover-actions button[data-nav-focus] {
  box-shadow: 0 0 0 2px var(--focus-ring);
}
.cover-actions {
  position: absolute;
  left: 8px;
  right: 8px;
  bottom: 8px;
  display: flex;
  gap: 6px;
  opacity: 0;
  transition: opacity 150ms;
}
.frame:hover .cover-actions,
.frame:focus-within .cover-actions,
.frame.open .cover-actions {
  opacity: 1;
}
.cover-actions button {
  height: 36px;
  border: 0;
  border-radius: 8px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
/* «Gioca»: the light button (board L01). */
.play {
  flex: 1;
  min-width: 0;
  background: var(--ink);
  color: var(--bg);
  font-weight: 600;
}
.more-wrap {
  position: relative;
}
.more {
  width: 36px;
  background: color-mix(in srgb, var(--surface-raised) 90%, transparent);
  color: var(--ink);
}
/* E01: not reachable, shown faded and not launchable. */
/* (On the cover, not the frame: the frame keeps the focus ring.) */
.card.off .play {
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
.name {
  min-width: 0;
  font-weight: 500;
  overflow-wrap: anywhere;
}
</style>
