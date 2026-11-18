<script setup lang="ts">
// Game details (#34, boards «Dettagli» L04–L10 desktop and «Tel-Dettagli» phone portrait), inside
// the Library section. L04 on top: «‹ Libreria», cover, console, title, play time and last game,
// «Riprendi» (accent; the newest of the auto quick save and the slots, as «Continua a giocare»)
// or «Gioca» when nothing is saved, the favourite and «…» (L12 «Scegli la copertina…», which is
// reachable from here only, Cowork #40; hide). Under it six tabs: L05 Salvataggi, L06
// Impostazioni del gioco, L07 Controlli, L08 Codici trucco and L09 Patch e traduzioni (not built
// yet: the reason, X05), L10 File e informazioni.
// Phone in portrait (device phone, as the quick menu tells it): ‹ and «…» on top, the cover beside
// the title, «Riprendi» full width, the tabs as scrolling pills.
//
// Controller / keyboard: the shell's nav scope covers it; L1 / R1 change tab (exposed `tab`);
// back (B, Esc) returns to the library on this game's cover (the shell does it).
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft, Ellipsis, EllipsisVertical, Heart, Play } from '@lucide/vue'
import { emu, type GameId, type LibraryGame, type SettingsSchema } from '../emu'
import { resumePoint, type ResumePoint } from '../gameDetails'
import { errorText, playTimeLabel, saveWhenLabel } from '../i18n/labels'
import { library } from '../library'
import { activeScopeRoot } from '../nav'
import { loadSchema } from '../settingsModel'
import CoverMenu from './CoverMenu.vue'
import CoverPicker from './CoverPicker.vue'
import DetailsControls from './DetailsControls.vue'
import DetailsFiles from './DetailsFiles.vue'
import DetailsSaves from './DetailsSaves.vue'
import DetailsSettings from './DetailsSettings.vue'
import GameCover from './GameCover.vue'

const props = defineProps<{ id: GameId }>()
const emit = defineEmits<{
  back: []
  /** `slot`: the quick save to load once the game runs. */
  play: [game: LibraryGame, slot?: number]
  /** L05 «Mostra i backup»: the Saves section on this game. */
  backups: [LibraryGame]
  /** L05 «Importa…»: the Saves section's import, for this game. */
  import: [LibraryGame]
  /** L06: the Settings section at «Solo questo gioco». */
  settings: [LibraryGame]
  /** L07: button remapping (C02) at «Solo questo gioco». */
  remap: [LibraryGame]
  error: [string]
  notice: [string]
}>()

const { t } = useI18n()

const game = computed(() => library.games.find(g => g.id === props.id) ?? null)
// The game left the library (its folder removed): back to the grid.
watch(game, g => {
  if (!g && library.loaded) emit('back')
})

// ---- Phone in portrait ----
const schema = ref<SettingsSchema | null>(null)
const portraitQuery = window.matchMedia('(orientation: portrait)')
const portrait = ref(portraitQuery.matches)
const onOrientation = (e: MediaQueryListEvent) => (portrait.value = e.matches)
portraitQuery.addEventListener('change', onOrientation)
const phone = computed(() => schema.value?.device === 'phone' && portrait.value)

// ---- L04: «Riprendi» / «Gioca» ----
const resume = ref<ResumePoint | null>(null)
async function loadResume() {
  const g = game.value
  if (!g) return
  try {
    const rows = await emu.settings.page('saves', { level: 'global' })
    const slots = Number(rows.find(r => r.key === 'saves.slots')?.value ?? 10) || 10
    resume.value = resumePoint(await emu.saves.game(g.path, slots))
  } catch {
    // Saves unreadable for now: the game starts as «Gioca» would.
    resume.value = null
  }
}
watch(() => `${game.value?.path}:${game.value?.lastPlayed}`, () => void loadResume())

function play() {
  const g = game.value
  if (!g) return
  if (!g.available) {
    emit('error', errorText({ code: 'libraryGameUnavailable', params: { id: g.id } }))
    return
  }
  emit('play', g, resume.value?.slot ?? undefined)
}

const meta = computed(() => {
  const g = game.value
  if (!g || g.lastPlayed === null) return t('library.neverPlayed')
  return t(phone.value ? 'details.metaShort' : 'details.meta', {
    time: playTimeLabel(g.playTimeSecs, true),
    when: saveWhenLabel(g.lastPlayed),
  })
})

async function setFlags(flags: { favourite?: boolean; hidden?: boolean }) {
  const g = game.value
  if (!g) return
  try {
    await emu.library.setFlags(g.id, flags)
  } catch (e) {
    emit('error', errorText(e))
  }
}

// ---- «…» and L12 ----
const menuOpen = ref(false)
const picking = ref(false)
const moreButton = ref<HTMLButtonElement | null>(null)
function closeMenu() {
  menuOpen.value = false
  void nextTick(() => moreButton.value?.focus())
}
function menuFlags(flags: { favourite?: boolean; hidden?: boolean }) {
  closeMenu()
  void setFlags(flags)
}
function chooseCover() {
  menuOpen.value = false
  picking.value = true
}
function closeCover() {
  picking.value = false
  void nextTick(() => moreButton.value?.focus())
}

// ---- Tabs ----
const TABS = ['saves', 'settings', 'controls', 'cheats', 'patches', 'files'] as const
type Tab = (typeof TABS)[number]
const current = ref<Tab>('saves')
const tabList = ref<HTMLElement | null>(null)
function tabLabel(tab: Tab): string {
  return t(phone.value && tab === 'settings' ? 'details.tabs.settingsShort' : `details.tabs.${tab}`)
}
/** L1 / R1: the previous / next tab; the focus follows when it was on the tabs. */
function step(dir: -1 | 1) {
  const i = TABS.indexOf(current.value)
  current.value = TABS[(i + dir + TABS.length) % TABS.length]!
  void nextTick(() => {
    const list = tabList.value
    if (!list) return
    const on = list.querySelector<HTMLElement>('[aria-selected="true"]')
    on?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
    if (list.contains(document.activeElement)) on?.focus()
  })
}
defineExpose({ tab: step })

// ---- Mount: the focus on «Riprendi» (or the way back), Esc goes back ----
const root = ref<HTMLElement | null>(null)
function onKey(e: KeyboardEvent) {
  // Only when no menu or dialog of this page has its own scope open (they close on Esc).
  if (e.code !== 'Escape' || !root.value) return
  if (!activeScopeRoot()?.contains(root.value)) return
  emit('back')
}
onMounted(async () => {
  window.addEventListener('keydown', onKey)
  await nextTick()
  root.value?.querySelector<HTMLElement>('[data-nav-initial]:not(:disabled)')?.focus()
  schema.value = await loadSchema()
  await loadResume()
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey)
  portraitQuery.removeEventListener('change', onOrientation)
})
</script>

<template>
  <div
    v-if="game"
    ref="root"
    class="details"
    :class="{ phone }"
    :style="{ '--tint': game.colour }"
  >
    <!-- L04 -->
    <section class="hero" :aria-label="game.title">
      <div v-if="phone" class="phone-bar">
        <button
          class="icon-btn"
          :aria-label="t('details.back')"
          :title="t('details.back')"
          @click="emit('back')"
        >
          <ChevronLeft :size="24" :stroke-width="1.75" aria-hidden="true" />
        </button>
        <div class="more-wrap">
          <button
            ref="moreButton"
            class="icon-btn"
            :aria-label="t('library.more')"
            :title="t('library.more')"
            aria-haspopup="menu"
            :aria-expanded="menuOpen"
            @click="menuOpen = !menuOpen"
          >
            <EllipsisVertical :size="22" :stroke-width="1.75" aria-hidden="true" />
          </button>
          <CoverMenu
            v-if="menuOpen"
            :game="game"
            :items="['cover', 'favourite', 'hide']"
            @cover="chooseCover"
            @flags="menuFlags"
            @close="closeMenu"
          />
        </div>
      </div>
      <button v-else class="back-link" @click="emit('back')">
        <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
        {{ t('shell.library') }}
      </button>

      <div class="hero-row">
        <GameCover class="hero-cover" :game="game" :badge="false" />
        <div class="hero-text">
          <div class="console">{{ t(`console.${game.console}`) }}</div>
          <h1 class="title">{{ game.title }}</h1>
          <div class="meta num">{{ meta }}</div>
          <div v-if="!phone" class="actions">
            <button
              class="btn btn-primary play"
              data-nav-initial
              :aria-disabled="!game.available || undefined"
              @click="play"
            >
              <Play :size="16" fill="currentColor" :stroke-width="0" aria-hidden="true" />
              {{ t(resume?.none ? 'library.hero.play' : 'library.hero.resume') }}
            </button>
            <button class="btn tall" @click="setFlags({ favourite: !game.favourite })">
              <Heart
                :size="16"
                :stroke-width="1.75"
                :fill="game.favourite ? 'currentColor' : 'none'"
                aria-hidden="true"
              />
              {{ t(game.favourite ? 'library.menu.unfavourite' : 'library.menu.favourite') }}
            </button>
            <div class="more-wrap">
              <button
                ref="moreButton"
                class="btn tall"
                :aria-label="t('library.more')"
                :title="t('library.more')"
                aria-haspopup="menu"
                :aria-expanded="menuOpen"
                @click="menuOpen = !menuOpen"
              >
                <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
              </button>
              <CoverMenu
                v-if="menuOpen"
                :game="game"
                :items="['cover', 'hide']"
                @cover="chooseCover"
                @flags="menuFlags"
                @close="closeMenu"
              />
            </div>
          </div>
          <p v-if="!game.available" class="unavailable small">
            {{ t('errors.libraryGameUnavailable') }}
          </p>
        </div>
      </div>
      <button
        v-if="phone"
        class="btn btn-primary play wide"
        data-nav-initial
        :aria-disabled="!game.available || undefined"
        @click="play"
      >
        <Play :size="18" fill="currentColor" :stroke-width="0" aria-hidden="true" />
        {{ t(resume?.none ? 'library.hero.play' : 'library.hero.resume') }}
      </button>
    </section>

    <div ref="tabList" class="tabs" role="tablist" :aria-label="t('details.tabsLabel')">
      <button
        v-for="tab in TABS"
        :key="tab"
        class="tab"
        :class="{ on: tab === current }"
        role="tab"
        :aria-selected="tab === current"
        @click="current = tab"
      >
        {{ tabLabel(tab) }}
      </button>
    </div>

    <div class="panel" role="tabpanel" :aria-label="tabLabel(current)">
      <DetailsSaves
        v-if="current === 'saves'"
        :game="game"
        :phone="phone"
        @play="slot => emit('play', game!, slot)"
        @backups="emit('backups', game!)"
        @import="emit('import', game!)"
        @changed="loadResume"
        @error="text => emit('error', text)"
        @notice="text => emit('notice', text)"
      />
      <DetailsSettings
        v-else-if="current === 'settings'"
        :game="game"
        :phone="phone"
        @open="emit('settings', game!)"
        @error="text => emit('error', text)"
      />
      <DetailsControls
        v-else-if="current === 'controls'"
        :game="game"
        :phone="phone"
        @remap="emit('remap', game!)"
        @error="text => emit('error', text)"
      />
      <DetailsFiles
        v-else-if="current === 'files'"
        :game="game"
        :phone="phone"
        @cover="picking = true"
        @error="text => emit('error', text)"
      />
      <!-- L08, L09: not built yet (X05, the reason). -->
      <div v-else class="not-yet">
        <p class="lead">{{ t(`details.${current}.lead`) }}</p>
        <p class="muted">{{ t(`details.${current}.notYet`) }}</p>
      </div>
    </div>

    <CoverPicker v-if="picking" :game="game" @close="closeCover" />
  </div>
</template>

<style scoped>
.details {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}
.muted {
  margin: 0;
  color: var(--ink-muted);
}
.small {
  font-size: 12px;
  line-height: 16px;
}
.btn.tall {
  height: 40px;
}
.icon-btn {
  width: 44px;
  height: 44px;
  border: 0;
  border-radius: var(--radius-10);
  background: none;
  display: grid;
  place-items: center;
}
.icon-btn:hover {
  background: var(--surface-hover);
}
.more-wrap {
  position: relative;
}

/* L04: the cover's colour fades down behind the header (board «Dettagli»). */
.hero {
  position: relative;
  padding: 24px 32px 28px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  background: linear-gradient(
    180deg,
    color-mix(in srgb, var(--tint) 33%, transparent) 0%,
    transparent 100%
  );
}
.back-link {
  align-self: flex-start;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 0;
  border: 0;
  background: none;
  color: var(--ink-muted);
  font-weight: 500;
}
.back-link:hover {
  color: var(--ink);
}
.hero-row {
  display: flex;
  flex-wrap: wrap;
  gap: 28px;
  align-items: flex-end;
}
.hero-cover {
  flex: 0 0 auto;
  width: 168px;
  box-shadow:
    0 0 60px color-mix(in srgb, var(--tint) 66%, transparent),
    0 24px 64px #0000008c;
}
.hero-text {
  flex: 1 1 420px;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.console {
  font-size: 11px;
  line-height: 14px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--ink-muted);
}
.title {
  margin: 0;
  font-size: 32px;
  line-height: 38px;
  font-weight: 600;
  letter-spacing: -0.02em;
  overflow-wrap: anywhere;
}
.meta {
  color: var(--ink-muted);
  font-size: 15px;
  line-height: 22px;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 6px;
}
.play[aria-disabled] {
  opacity: 0.4;
  cursor: not-allowed;
}
.unavailable {
  margin: 0;
  color: var(--warning);
}

/* Tabs: underlined (desktop). */
.tabs {
  display: flex;
  gap: 4px;
  padding: 0 32px;
  border-bottom: 1px solid var(--line);
  overflow-x: auto;
}
.tab {
  flex: 0 0 auto;
  height: 44px;
  padding: 0 12px;
  border: 0;
  background: none;
  font-weight: 500;
  color: var(--ink-muted);
  white-space: nowrap;
}
.tab:hover {
  color: var(--ink);
}
.tab.on {
  color: var(--ink);
  box-shadow: inset 0 -2px 0 var(--accent-text);
}
.panel {
  padding: 24px 32px 48px;
  max-width: 1040px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.not-yet {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.lead {
  margin: 0;
}

/* Phone in portrait (board «Tel-Dettagli», 390 × 844). */
.details.phone {
  font-size: 15px;
  line-height: 22px;
}
.phone .hero {
  padding: 8px 16px 0;
  gap: 16px;
}
.phone-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 0 -8px;
}
.phone .hero-row {
  flex-wrap: nowrap;
  gap: 14px;
}
.phone .hero-cover {
  width: 96px;
  box-shadow: 0 0 40px color-mix(in srgb, var(--tint) 66%, transparent);
}
.phone .hero-text {
  flex: 1;
  gap: 4px;
}
.phone .console {
  font-size: 12px;
  line-height: 16px;
}
.phone .title {
  font-size: 22px;
  line-height: 28px;
}
.phone .meta {
  font-size: 13px;
  line-height: 18px;
}
.play.wide {
  height: 52px;
  border-radius: var(--radius-10);
  font-size: 16px;
  gap: 8px;
}
/* Tabs: pills that scroll sideways (phone). */
.phone .tabs {
  gap: 8px;
  padding: 16px 16px 0;
  border-bottom: 0;
}
.phone .tab {
  height: 36px;
  padding: 0 14px;
  border-radius: 999px;
  font-size: 14px;
  background: var(--surface-raised);
  color: var(--ink);
  border: 1px solid var(--line);
}
.phone .tab.on {
  box-shadow: none;
  background: var(--accent-soft);
  color: var(--accent-text);
  border-color: color-mix(in srgb, var(--violet) 40%, transparent);
}
.phone .panel {
  padding: 16px;
  gap: 16px;
}
</style>
