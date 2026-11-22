<script setup lang="ts">
// The app shell outside the game (design L01, desktop mode): top bar with the wordmark,
// rail with the five sections, the current section on the right. Hidden while a game is open:
// App.vue keeps the route, so closing the game comes back to it.
//
// Where the shell is, is one typed `Route` (route.ts), held by App.vue: each move is one of its
// pure functions, and each section gets what it is opened for as props, read when it mounts;
// once it has mounted the shell drops that arrival (`arrived`).
//
// Controller / keyboard: one nav scope over the whole shell. Moving onto a rail item does not
// switch section, confirming it does; back from the content returns the focus to the rail. A
// section with steps of its own (the settings levels, the controls tabs and their way back up, a
// game's details) registers them itself (nav.ts `useNavSection`); what it leaves comes here.
//
// Game details (L04–L10, #34) live in the Library section: shown instead of the grid while the
// route names a game (a cover's click, right click or X; «Dettagli del gioco» in Saves).
// Back (B, Esc, «‹ Libreria», the rail's Library) returns to the grid on that game's cover.
// From there a game opens, and closing it comes back to the details; «Mostra i backup» /
// «Importa…» (L05), «Apri tutte le impostazioni…» (L06) and «Rimappa…» (L07) jump to the
// Saves, Settings and Controls sections for that one game.
//
// The search everywhere (X01, #32): the top bar's field, Ctrl + K, or Y on the controller (the
// `search` step, which a dialog or a menu open above the shell stops) open SearchPalette; a
// setting chosen there opens its page with the row lit (`settingRowFor`), a game its details.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { uiMode } from '@/composables/useUiMode'
import { cn } from '@/lib/cn'
import { activeScopeRoot, useNavScope } from '@/nav'
import type { LibraryGame, SettingKey } from '@/emu'
import ControlsView from '@/components/controls/ControlsView.vue'
import GameDetails from '@/components/details/GameDetails.vue'
import LibraryView from '@/components/library/LibraryView.vue'
import SavesView from '@/components/saves/SavesView.vue'
import SearchPalette from '@/components/search/SearchPalette.vue'
import SettingsView from '@/components/settings/SettingsView.vue'
import AppRail from './AppRail.vue'
import AppTopBar from './AppTopBar.vue'
import SectionPlaceholder from './SectionPlaceholder.vue'
import TvButtonBar from './TvButtonBar.vue'
import TvTabs from './TvTabs.vue'
import { tvHints } from './tvHints'
import {
  arrivalFor,
  arrived,
  closeDetails,
  openDetails,
  openSection,
  pickSection,
  remapFor,
  savesFor,
  settingRowFor,
  settingsFor,
  showingDetails,
  type Route,
} from './route'
import { hasOwnInitialFocus, isFlush, isTvLaidOut, sectionStep } from './shellLayout'
import { useCoverHaze } from './useCoverHaze'

const emit = defineEmits<{
  openFile: []
  /** `slot`: a quick save to load once the game runs (Saves, V03). */
  openGame: [game: LibraryGame, slot?: number]
}>()
const route = defineModel<Route>({ required: true })

const section = computed(() => route.value.section)
const details = computed(() => (showingDetails(route.value) ? route.value.details : null))
const hazeStyle = useCoverHaze(section)

const root = ref<HTMLElement | null>(null)
const rail = ref<InstanceType<typeof AppRail> | null>(null)
// TV mode (#30): the tabs at the top instead of the rail, L1 / R1 switch section from anywhere
// (a section's own L1 / R1, the settings levels or the controls tabs, come first).
const tabs = ref<InstanceType<typeof TvTabs> | null>(null)
function stepSection(dir: -1 | 1) {
  if (!uiMode.tv) return false
  const onTabs = tabs.value?.hasFocus() ?? false
  go(pickSection(route.value, sectionStep(section.value, dir)))
  if (onTabs) void nextTick(() => tabs.value?.focusCurrent())
}
const searching = ref(false)
const openSearch = () => (searching.value = true)
useNavScope(root, {
  autofocus: true,
  back: () => (uiMode.tv ? tabs.value : rail.value)?.focusCurrent(),
  tab: stepSection,
  search: openSearch,
})

/** Ctrl + K (⌘ K): the search, unless a dialog or a menu above the shell has the steps. */
function onKey(e: KeyboardEvent) {
  if (e.code !== 'KeyK' || !(e.ctrlKey || e.metaKey) || e.altKey || e.shiftKey) return
  if (activeScopeRoot() !== root.value) return
  e.preventDefault()
  openSearch()
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))

// A setting chosen in the search: the Settings section mounts again for it, even when shown.
const settingsVisit = ref(0)
function openSetting(key: SettingKey) {
  settingsVisit.value++
  go(settingRowFor(route.value, key))
}

// The section on screen has read its arrival in its setup: drop it, after the render.
watch(
  route,
  r => {
    const next = arrived(r)
    if (next !== r) route.value = next
  },
  { flush: 'post' },
)

const go = (to: Route) => (route.value = to)
</script>

<template>
  <div ref="root" class="relative isolate flex h-full flex-col overflow-hidden bg-bg">
    <div class="ambient-haze" aria-hidden="true" :style="hazeStyle"></div>
    <SearchPalette
      v-model:open="searching"
      @open-setting="openSetting"
      @details="g => go(openDetails(g.id))"
      @play="g => emit('openGame', g)"
    />
    <AppTopBar v-if="!uiMode.tv" @search="openSearch" />
    <TvTabs
      v-else
      ref="tabs"
      :current="section"
      :content-has-initial="hasOwnInitialFocus(section)"
      @pick="s => go(pickSection(route, s))"
    />
    <div class="flex min-h-0 flex-1">
      <AppRail
        v-if="!uiMode.tv"
        ref="rail"
        :current="section"
        :content-has-initial="hasOwnInitialFocus(section)"
        @pick="s => go(pickSection(route, s))"
      />
      <main
        :class="
          cn(
            'flex min-w-0 flex-1 flex-col',
            isFlush(section, details !== null, uiMode.tv)
              ? 'overflow-hidden p-0'
              : 'overflow-auto p-32',
            // TV mode: the sections not laid out for it yet (Saves, Controls, Cloud, Settings, a
            // game's details) keep the desk's layout, scaled up as it is.
            !isTvLaidOut(section, details !== null) && 'tv:zoom-tv',
          )
        "
      >
        <GameDetails
          v-if="details"
          :id="details"
          :key="details"
          @back="go(closeDetails(route))"
          @play="(g, slot) => emit('openGame', g, slot)"
          @backups="g => go(savesFor(route, g.id))"
          @import="g => go(savesFor(route, g.id, true))"
          @settings="g => go(settingsFor(route, g))"
          @remap="g => go(remapFor(route, g))"
        />
        <LibraryView
          v-else-if="section === 'library'"
          :return-to="arrivalFor(route, 'library')?.cover ?? null"
          @open="(g, slot) => emit('openGame', g, slot)"
          @open-saves="g => go(savesFor(route, g.id))"
          @details="g => go(openDetails(g.id))"
          @open-file="emit('openFile')"
        />
        <SavesView
          v-else-if="section === 'saves'"
          :arrival="arrivalFor(route, 'saves')"
          @open-game="(g, slot) => emit('openGame', g, slot)"
          @open-details="g => go(openDetails(g.id))"
          @open-library="go(openSection(route, 'library'))"
        />
        <SettingsView
          v-else-if="section === 'settings'"
          :key="settingsVisit"
          :game="arrivalFor(route, 'settings')?.game ?? null"
          :row="arrivalFor(route, 'settings')?.row ?? null"
          @open-controls="go(openSection(route, 'controls'))"
        />
        <ControlsView
          v-else-if="section === 'controls'"
          :game="arrivalFor(route, 'controls')?.game ?? null"
        />
        <SectionPlaceholder v-else :section="section" />
      </main>
    </div>
    <TvButtonBar v-if="uiMode.tv" :hints="tvHints(section, details !== null)" />
  </div>
</template>
