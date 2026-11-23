<script setup lang="ts">
// The app shell outside the game (design L01, desktop mode): top bar with the wordmark,
// rail with the five sections, the current section on the right. Hidden while a game is open:
// App.vue keeps the route, so closing the game comes back to it.
//
// The phone (#29): held upright the sections are a tab bar at the bottom (PhoneTabBar); on its
// side the rail is a column on the camera's edge, the top bar and the section beside it. The
// whole shell keeps clear of the system's bars and the notch (the `safe` spacing tokens): the top
// bar and the side column reach under the status bar, their glass behind it and their content
// below it (#29: no strip of the page between the status bar and the bar). Under
// the top bar, E13's warning when the phone is almost full (storage/StorageWarning).
//
// Where the shell is, is one typed `Route` (route.ts), held by App.vue: each move is one of its
// pure functions, and each section gets what it is opened for as props, read when it mounts;
// once it has mounted the shell drops that arrival (`arrived`).
//
// Controller / keyboard: one nav scope over the whole shell. Moving onto a rail item does not
// switch section, confirming it does; back from the content returns the focus to the rail (on
// the phone, Android's back and B go to the section left last, then the Library, then leave to
// the background: route.ts `backFrom`). A
// section with steps of its own (the settings levels, the controls tabs and their way back up, a
// game's details) registers them itself (nav.ts `useNavSection`); what it leaves comes here.
//
// Game details (L04–L10, #34) live in the Library section: shown instead of the grid while the
// route names a game (a cover's click, right click or X; «Dettagli del gioco» in Saves).
// Back (B, Esc, «‹ Libreria», the rail's Library) returns to the grid on that game's cover.
// A rail item or tab always opens its section at its root (#127): the Library's grid even with
// details left behind another section, and the section already shown mounted again (a Settings
// page, a Saves panel on the phone close).
// From there a game opens, and closing it comes back to the details; «Mostra i backup» /
// «Importa…» (L05), «Apri tutte le impostazioni…» (L06) and «Rimappa…» (L07) jump to the
// Saves, Settings and Controls sections for that one game.
//
// The search everywhere (X01, #32): the shell is the search's host while it shows
// (search/useSearch.ts; the palette is App.vue's). The top bar's field, Ctrl + K, or Y on the
// controller (the `search` step, which a dialog or a menu open above the shell stops) open it; a
// setting chosen there opens its page with the row lit (`settingRowFor`), a game its details
// (Ctrl + Enter starts it), and of the actions only fullscreen runs (the others need a game).
import { computed, nextTick, ref, watch } from 'vue'
import { uiMode } from '@/composables/useUiMode'
import { cn } from '@/design-system/lib/cn'
import { ignore, report } from '@/lib/errors'
import { useNavScope } from '@/nav'
import { emu, type LibraryGame, type SettingKey } from '@/emu'
import CloudView from '@/components/cloud/CloudView.vue'
import ControlsView from '@/components/controls/ControlsView.vue'
import GameDetails from '@/components/details/GameDetails.vue'
import LibraryView from '@/components/library/LibraryView.vue'
import SavesView from '@/components/saves/SavesView.vue'
import { SHELL } from '@/components/search/searchContext'
import { useSearchHost } from '@/components/search/useSearch'
import SettingsView from '@/components/settings/SettingsView.vue'
import StorageWarning from '@/components/storage/StorageWarning.vue'
import UpdateReadyCard from '@/components/updates/UpdateReadyCard.vue'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import { ShellNav } from '@/lib/phoneLayout'
import AppRail from './AppRail.vue'
import AppTopBar from './AppTopBar.vue'
import PhoneTabBar from './PhoneTabBar.vue'
import TvButtonBar from './TvButtonBar.vue'
import TvTabs from './TvTabs.vue'
import { tvHints } from './tvHints'
import {
  arrivalFor,
  arrived,
  backFrom,
  closeDetails,
  openDetails,
  openSection,
  pickSection,
  remapFor,
  savesFor,
  settingRowFor,
  moved,
  settingsFor,
  showingDetails,
  type Route,
} from './route'
import type { Section } from './sections'
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
// Where the sections sit (#29): the desktop's rail, the phone's tab bar (upright) or the rail as
// a column on the camera's edge (on its side). TV mode's tabs come before all of them.
const layout = useDeviceLayout()
const nav = computed(() => (uiMode.tv ? null : layout.nav.value))
const rail = ref<InstanceType<typeof AppRail> | null>(null)
const tabBar = ref<InstanceType<typeof PhoneTabBar> | null>(null)
/** The phone's room for the system's bars and the notch, around the whole shell. */
const shellClass = computed(() => {
  switch (nav.value) {
    case ShellNav.Tabs:
      return 'flex-col'
    case ShellNav.Column:
      return layout.camera.value === 'left'
        ? 'flex-row pr-safe-right'
        : 'flex-row-reverse pl-safe-left'
    default:
      return 'flex-col'
  }
})
// TV mode (#30): the tabs at the top instead of the rail, L1 / R1 switch section from anywhere
// (a section's own L1 / R1, the settings levels or the controls tabs, come first).
const tabs = ref<InstanceType<typeof TvTabs> | null>(null)
function stepSection(dir: -1 | 1) {
  if (!uiMode.tv) return false
  const onTabs = tabs.value?.hasFocus() ?? false
  go(pickSection(route.value, sectionStep(section.value, dir)))
  if (onTabs) void nextTick(() => tabs.value?.focusCurrent())
}
const { openSearch } = useSearchHost({
  root,
  context: () => SHELL,
  // Outside a game only the keyboard's fullscreen runs (the others carry a reason).
  action: id => {
    if (id === 'fullscreen') report(emu.toggleFullscreen())
  },
  setting: key => openSetting(key),
  game: (g, start) => (start ? emit('openGame', g) : go(openDetails(route.value, g.id))),
})
/**
 * The phone's back at the shell's root (Android's back button or gesture, B; #29): the section
 * left last, then the Library, then the app to the background, as at any app's first screen.
 */
function phoneBack() {
  const to = backFrom(route.value)
  if (to) route.value = to
  else ignore(emu.leaveToBackground(), 'back at the shell root: the app stays in front')
}
useNavScope(root, {
  autofocus: true,
  back: () => {
    if (layout.touchDensity.value && !uiMode.tv) return phoneBack()
    ;(uiMode.tv ? tabs.value : (rail.value ?? tabBar.value))?.focusCurrent()
  },
  tab: stepSection,
  search: openSearch,
})

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

/** A move: the section left goes onto the trail (route.ts `moved`). */
const go = (to: Route) => (route.value = moved(route.value, to))

/** Mounts of the section shown by picking it again: it starts again at its root. */
const visits = ref(0)
/** A rail item or tab: the section at its root (route.ts `pickSection`). */
function pick(s: Section) {
  if (s === section.value && details.value === null) visits.value++
  go(pickSection(route.value, s))
}
</script>

<template>
  <div ref="root" :class="cn('relative isolate flex h-full overflow-hidden bg-bg', shellClass)">
    <!-- The cover-coloured haze is the desktop's and TV's: the phone keeps the plain graphite
         (owner, #105: the wash behind the top bar does not belong to the app's look there). -->
    <div
      v-if="!layout.touchDensity.value"
      class="ambient-haze"
      aria-hidden="true"
      :style="hazeStyle"
    ></div>
    <AppRail
      v-if="nav === ShellNav.Column"
      ref="rail"
      :edge="layout.camera.value"
      :current="section"
      :content-has-initial="hasOwnInitialFocus(section)"
      @pick="pick"
    />
    <div class="flex min-h-0 min-w-0 flex-1 flex-col">
      <AppTopBar v-if="!uiMode.tv" @search="openSearch" />
      <!-- E13 (#29): the phone almost full, under the top bar. -->
      <StorageWarning v-if="layout.touchDensity.value && !uiMode.tv" />
      <!-- TV mode only: not the `v-else` of the warning above, or the desktop shows them too. -->
      <TvTabs
        v-if="uiMode.tv"
        ref="tabs"
        :current="section"
        :content-has-initial="hasOwnInitialFocus(section)"
        @pick="pick"
      />
      <div class="flex min-h-0 flex-1">
        <AppRail
          v-if="nav === ShellNav.Rail"
          ref="rail"
          :current="section"
          :content-has-initial="hasOwnInitialFocus(section)"
          @pick="pick"
        />
        <!-- A section (or a game's details) coming in fades in with 4 px of rise (motion.css
             `enter-rise`): keyed by what it shows, so each change mounts it again and plays it;
             `visits`: the section shown, picked again, mounts again at its root.
             «Riduci le animazioni» cuts it to its last frame (styles.css). -->
        <main
          :key="`${details ?? section}:${visits}`"
          :class="
            cn(
              'flex min-w-0 flex-1 flex-col animate-[enter-rise_var(--transition-duration-fast)_var(--ease-out)]',
              isFlush(section, details !== null, uiMode.tv)
                ? 'overflow-hidden p-0'
                : 'overflow-auto p-16',
              // TV mode: the sections not laid out for it yet (Saves, Controls, Cloud, a game's
              // details) keep the desk's layout, scaled up as it is.
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
            @details="g => go(openDetails(route, g.id))"
            @open-file="emit('openFile')"
          />
          <SavesView
            v-else-if="section === 'saves'"
            :arrival="arrivalFor(route, 'saves')"
            @open-game="(g, slot) => emit('openGame', g, slot)"
            @open-details="g => go(openDetails(route, g.id))"
            @open-library="go(openSection(route, 'library'))"
          />
          <SettingsView
            v-else-if="section === 'settings'"
            :key="settingsVisit"
            :game="arrivalFor(route, 'settings')?.game ?? null"
            :row="arrivalFor(route, 'settings')?.row ?? null"
            @open-controls="go(openSection(route, 'controls'))"
            @open-cloud="go(openSection(route, 'cloud'))"
          />
          <ControlsView
            v-else-if="section === 'controls'"
            :game="arrivalFor(route, 'controls')?.game ?? null"
          />
          <CloudView v-else />
        </main>
      </div>
      <PhoneTabBar
        v-if="nav === ShellNav.Tabs"
        ref="tabBar"
        :current="section"
        :content-has-initial="hasOwnInitialFocus(section)"
        @pick="pick"
      />
      <TvButtonBar v-if="uiMode.tv" :hints="tvHints(section, details !== null)" />
    </div>
    <!-- X06: a new version ready to install (inside the nav scope: reachable by the steps). -->
    <UpdateReadyCard />
  </div>
</template>
