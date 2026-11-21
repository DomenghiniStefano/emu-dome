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
import { computed, ref, watch } from 'vue'
import { cn } from '@/lib/cn'
import { useNavScope } from '@/nav'
import type { LibraryGame } from '@/emu'
import ControlsView from '@/components/controls/ControlsView.vue'
import GameDetails from '@/components/details/GameDetails.vue'
import LibraryView from '@/components/library/LibraryView.vue'
import SavesView from '@/components/saves/SavesView.vue'
import SettingsView from '@/components/settings/SettingsView.vue'
import AppRail from './AppRail.vue'
import AppTopBar from './AppTopBar.vue'
import SectionPlaceholder from './SectionPlaceholder.vue'
import {
  arrivalFor,
  arrived,
  closeDetails,
  openDetails,
  openSection,
  pickSection,
  remapFor,
  savesFor,
  settingsFor,
  showingDetails,
  type Route,
} from './route'
import { hasOwnInitialFocus, isFlush } from './shellLayout'
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
useNavScope(root, { autofocus: true, back: () => rail.value?.focusCurrent() })

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
    <AppTopBar />
    <div class="flex min-h-0 flex-1">
      <AppRail
        ref="rail"
        :current="section"
        :content-has-initial="hasOwnInitialFocus(section)"
        @pick="s => go(pickSection(route, s))"
      />
      <main
        :class="
          cn(
            'flex min-w-0 flex-1 flex-col',
            isFlush(section, details !== null) ? 'overflow-hidden p-0' : 'overflow-auto p-32',
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
          :game="arrivalFor(route, 'settings')?.game ?? null"
          @open-controls="go(openSection(route, 'controls'))"
        />
        <ControlsView
          v-else-if="section === 'controls'"
          :game="arrivalFor(route, 'controls')?.game ?? null"
        />
        <SectionPlaceholder v-else :section="section" />
      </main>
    </div>
  </div>
</template>
