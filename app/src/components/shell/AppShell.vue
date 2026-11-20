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
import { computed, ref } from 'vue'
import { cn } from '@/lib/cn'
import { useNavScope } from '@/nav'
import type { Section } from './sections'
import type { LibraryGame } from '@/emu'
import {
  controlsFocus,
  detailsOf,
  detailsReturn,
  gameScopeOf,
  settingsFocus,
} from '@/lib/gameDetails'
import { savesFocus, savesImportFor } from '@/lib/savesFocus'
import ControlsView from '@/components/controls/ControlsView.vue'
import GameDetails from '@/components/details/GameDetails.vue'
import LibraryView from '@/components/library/LibraryView.vue'
import SavesView from '@/components/saves/SavesView.vue'
import SettingsView from '@/components/settings/SettingsView.vue'
import AppRail from './AppRail.vue'
import AppTopBar from './AppTopBar.vue'
import SectionPlaceholder from './SectionPlaceholder.vue'
import { hasOwnInitialFocus, isFlush } from './shellLayout'
import { useCoverHaze } from './useCoverHaze'

defineProps<{ error: string | null }>()
const emit = defineEmits<{
  openFile: []
  /** `slot`: a quick save to load once the game runs (Saves, V03). */
  openGame: [game: LibraryGame, slot?: number]
  error: [string]
  notice: [string]
}>()
const section = defineModel<Section>({ required: true })

const hazeStyle = useCoverHaze(section)

const root = ref<HTMLElement | null>(null)
const rail = ref<InstanceType<typeof AppRail> | null>(null)

// L1 / R1 go to the section that has tabs or levels (the settings scope bar, the controls tabs).
const settingsView = ref<InstanceType<typeof SettingsView> | null>(null)
const controlsView = ref<InstanceType<typeof ControlsView> | null>(null)
const gameDetails = ref<InstanceType<typeof GameDetails> | null>(null)
useNavScope(root, {
  autofocus: true,
  // A section one step down (Controls C02–C04, a game's details) goes back up first.
  back: () => {
    if (showingDetails.value) return closeDetails()
    if (!controlsView.value?.back()) rail.value?.focusCurrent()
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
  savesFocus.value = g.id
  savesImportFor.value = g.id
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
  savesFocus.value = g.id
  section.value = 'saves'
}
</script>

<template>
  <div ref="root" class="relative isolate flex h-full flex-col overflow-hidden bg-bg">
    <div class="ambient-haze" aria-hidden="true" :style="hazeStyle"></div>
    <AppTopBar />
    <div class="flex min-h-0 flex-1">
      <AppRail
        ref="rail"
        :current="section"
        :content-has-initial="hasOwnInitialFocus(section)"
        @pick="pickSection"
      />
      <main
        :class="
          cn(
            'flex min-w-0 flex-1 flex-col',
            isFlush(section, showingDetails) ? 'overflow-hidden p-0' : 'overflow-auto p-32',
          )
        "
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
        <SectionPlaceholder v-else :section="section" />
      </main>
    </div>
  </div>
</template>
