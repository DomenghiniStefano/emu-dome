<script setup lang="ts">
// The app shell outside the game (design L01, desktop mode): top bar with the wordmark,
// rail with the five sections, the current section on the right. Hidden while a game is open:
// App.vue keeps the current section, so closing the game comes back to it.
//
// Controller / keyboard: one nav scope over the whole shell. Moving onto a rail item does not
// switch section, confirming it does; back from the content returns the focus to the rail.
//
// Game details (L04–L10, #34) live in the Library section: shown instead of the grid while
// `detailsOf` names a game (a cover's click, right click or X; «Dettagli del gioco» in Saves).
// Back (B, Esc, «‹ Libreria», the rail's Library) returns to the grid on that game's cover.
// From there a game opens, and closing it comes back to the details; «Mostra i backup» /
// «Importa…» (L05), «Apri tutte le impostazioni…» (L06) and «Rimappa…» (L07) jump to the
// Saves, Settings and Controls sections for that one game.
import { computed, ref, type Component } from 'vue'
import { useI18n } from 'vue-i18n'
import { Cloud, Gamepad2, Library, Save, Settings2 } from '@lucide/vue'
import { useNavScope } from '../nav'
import { SECTIONS, type Section } from '../sections'
import type { LibraryGame } from '../emu'
import { controlsFocus, detailsOf, detailsReturn, gameScopeOf, settingsFocus } from '../gameDetails'
import { savesFocus, savesImportFor } from '../savesFocus'
import ControlsView from './ControlsView.vue'
import GameDetails from './GameDetails.vue'
import LibraryView from './LibraryView.vue'
import SavesView from './SavesView.vue'
import SettingsView from './SettingsView.vue'

defineProps<{ error: string | null }>()
const emit = defineEmits<{
  openFile: []
  /** `slot`: a quick save to load once the game runs (Saves, V03). */
  openGame: [game: LibraryGame, slot?: number]
  error: [string]
  notice: [string]
}>()
const section = defineModel<Section>({ required: true })

const { t } = useI18n()

const ICONS: Record<Section, Component> = {
  library: Library,
  saves: Save,
  controls: Gamepad2,
  cloud: Cloud,
  settings: Settings2,
}
const root = ref<HTMLElement | null>(null)
const rail = ref<HTMLElement | null>(null)

function focusRail() {
  rail.value?.querySelector<HTMLElement>('[aria-current="page"]')?.focus()
}

// L1 / R1 go to the section that has tabs or levels (the settings scope bar, the controls tabs).
const settingsView = ref<InstanceType<typeof SettingsView> | null>(null)
const controlsView = ref<InstanceType<typeof ControlsView> | null>(null)
const gameDetails = ref<InstanceType<typeof GameDetails> | null>(null)
useNavScope(root, {
  autofocus: true,
  // A section one step down (Controls C02–C04, a game's details) goes back up first.
  back: () => {
    if (showingDetails.value) return closeDetails()
    if (!controlsView.value?.back()) focusRail()
  },
  tab: dir => {
    settingsView.value?.tab(dir)
    controlsView.value?.tab(dir)
    gameDetails.value?.tab(dir)
  },
})

// ---- Game details (L04–L10) ----
const showingDetails = computed(() => section.value === 'library' && detailsOf.value !== null)
function openDetails(g: LibraryGame) {
  detailsOf.value = g.id
  section.value = 'library'
}
/** Back to the grid, on the game's cover. */
function closeDetails() {
  detailsReturn.value = detailsOf.value
  detailsOf.value = null
}
function pickSection(s: Section) {
  // The rail's Library while the details are shown: back to the grid.
  if (s === 'library' && showingDetails.value) return closeDetails()
  section.value = s
}
/** L05 «Importa…»: the Saves section's import (V05), for this game. */
function importFor(g: LibraryGame) {
  savesFocus.value = g.path
  savesImportFor.value = g.path
  section.value = 'saves'
}
/** L06: the Settings section at «Solo questo gioco». */
function settingsFor(g: LibraryGame) {
  const scope = gameScopeOf(g)
  settingsFocus.value = scope ? { scope, title: g.title } : null
  section.value = 'settings'
}
/** L07: button remapping (C02) at «Solo questo gioco». */
function remapFor(g: LibraryGame) {
  const scope = gameScopeOf(g)
  controlsFocus.value = scope ? { scope, title: g.title } : null
  section.value = 'controls'
}

/** L01 «Salvataggi» on a game: the Saves section, on that game. */
function openSaves(g: LibraryGame) {
  savesFocus.value = g.path
  section.value = 'saves'
}

// The section's own first control takes the initial focus; a section without one
// (the placeholders) leaves it on its rail item.
const contentHasInitial = computed(() =>
  ['library', 'settings', 'controls', 'saves'].includes(section.value),
)
function initialOn(s: Section) {
  return s === section.value && !contentHasInitial.value ? '' : undefined
}
</script>

<template>
  <div ref="root" class="shell">
    <div class="haze" aria-hidden="true"></div>
    <header class="topbar">
      <div class="wordmark">EmuDome</div>
    </header>
    <div class="body">
      <nav ref="rail" class="rail" :aria-label="t('shell.sections')">
        <template v-for="s in SECTIONS" :key="s">
          <div v-if="s === 'settings'" class="spacer"></div>
          <button
            class="rail-item"
            :class="{ on: s === section }"
            :aria-current="s === section ? 'page' : undefined"
            :data-nav-initial="initialOn(s)"
            @click="pickSection(s)"
          >
            <component :is="ICONS[s]" :size="20" :stroke-width="1.75" aria-hidden="true" />
            <span class="rail-name">{{ t(`shell.${s}`) }}</span>
          </button>
        </template>
      </nav>
      <main
        class="content"
        :class="{
          flush:
            section === 'settings' ||
            section === 'controls' ||
            section === 'saves' ||
            showingDetails,
        }"
      >
        <GameDetails
          v-if="showingDetails && detailsOf"
          :id="detailsOf"
          ref="gameDetails"
          :key="detailsOf"
          @back="closeDetails"
          @play="(g, slot) => emit('openGame', g, slot)"
          @backups="openSaves"
          @import="importFor"
          @settings="settingsFor"
          @remap="remapFor"
          @error="text => emit('error', text)"
          @notice="text => emit('notice', text)"
        />
        <LibraryView
          v-else-if="section === 'library'"
          :error="error"
          @open="(g, slot) => emit('openGame', g, slot)"
          @open-saves="openSaves"
          @details="openDetails"
          @open-file="emit('openFile')"
          @error="text => emit('error', text)"
        />
        <SavesView
          v-else-if="section === 'saves'"
          @open-game="(g, slot) => emit('openGame', g, slot)"
          @open-details="openDetails"
          @open-library="section = 'library'"
          @error="text => emit('error', text)"
          @notice="text => emit('notice', text)"
        />
        <SettingsView
          v-else-if="section === 'settings'"
          ref="settingsView"
          @error="text => emit('error', text)"
          @open-controls="section = 'controls'"
        />
        <ControlsView
          v-else-if="section === 'controls'"
          ref="controlsView"
          @error="text => emit('error', text)"
        />
        <section v-else class="placeholder">
          <h1 class="title">{{ t(`shell.${section}`) }}</h1>
          <p class="line">{{ t('placeholder.line') }}</p>
        </section>
      </main>
    </div>
  </div>
</template>

<style scoped>
.shell {
  position: relative;
  isolation: isolate;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg);
  overflow: hidden;
}
/* Ambient haze (design L01): decoration only, off in Nero (--haze-opacity 0). */
.haze {
  position: absolute;
  inset: 0;
  z-index: -1;
  pointer-events: none;
  opacity: var(--haze-opacity);
  background:
    radial-gradient(60% 50% at 18% 0%, #2f5d8a 0%, transparent 70%),
    radial-gradient(40% 40% at 80% 10%, #1f6f78 0%, transparent 70%),
    radial-gradient(50% 50% at 60% 100%, #2b3a8c 0%, transparent 70%);
}
.topbar {
  flex: 0 0 40px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  background: var(--glass-thin);
  border-bottom: 1px solid var(--line);
  backdrop-filter: blur(24px) saturate(140%);
}
.wordmark {
  font-weight: 600;
  font-size: 14px;
  letter-spacing: -0.025em;
}
.body {
  flex: 1;
  min-height: 0;
  display: flex;
}
/* Rail (#59 item 1): 76 px, every item icon + name under it. */
.rail {
  flex: 0 0 76px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 12px 0;
  background: var(--glass-thin);
  border-right: 1px solid var(--line);
}
.spacer {
  flex: 1;
}
.rail-item {
  width: 64px;
  height: 52px;
  border: 0;
  border-radius: 8px;
  background: none;
  color: var(--ink-muted);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 3px;
  padding: 0;
  transition: background 150ms;
}
.rail-name {
  font-size: 10px;
  line-height: 12px;
  font-weight: 500;
  white-space: nowrap;
}
.rail-item:hover {
  background: var(--surface-hover);
}
.rail-item.on {
  background: var(--accent-soft);
  color: var(--accent-text);
  box-shadow: inset 2px 0 0 var(--violet);
}
.rail-item.on svg {
  filter: drop-shadow(0 0 6px color-mix(in srgb, var(--violet) 60%, transparent));
}
.content {
  flex: 1;
  min-width: 0;
  overflow: auto;
  padding: 32px;
  display: flex;
  flex-direction: column;
}
/* Settings: the category list sits flush and each column scrolls on its own. */
.content.flush {
  padding: 0;
  overflow: hidden;
}
.placeholder {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.title {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
  letter-spacing: -0.01em;
}
.line {
  margin: 0;
  color: var(--ink-muted);
}
</style>
