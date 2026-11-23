<script setup lang="ts">
// The quick menu (boards «MenuRapido3» G07–G11, G16, G17 and «Tel-MenuRapido3», 18/11).
// Desktop: a glass panel in the centre, 640 px wide and as tall as its content; the game is
// paused behind it, dimmed. Top to bottom: quick saves (Automatico + 1–4, «Altri slot e
// salvataggi» = G08), speed (Normale / Avanti veloce / Rallentatore, «Imposta le velocità» = G16),
// tools, the rows «Schermi e immagine» (G09: the menu becomes a bar at the bottom and the game
// stays visible above it) and «Controlli» (G10), then «Chiudi il gioco» and «Riprendi il gioco».
// NDS / 3DS: swap and the lid among the tools, the layout card first on G09, and from there the
// C09 screens editor over the whole game (Trello #28).
// The phone (owner, 2026-11-22, proposal «A»): a basic first level (PhoneHome: «Salva rapido»,
// «Carica rapido» on the quick slot, «Riprendi», the strip with «Chiudi il gioco», #121) and a second level «Altro» growing
// the same sheet (PhoneMore; «Altro › Salvataggi» is PhoneSaves, the pages G16 / G09 / G10 as
// on desktop). In portrait a bottom sheet under the game, as tall as its content, no handle; on
// its side (#29) a 300 px panel at the right edge, the game beside it. A tap on the game resumes;
// B / Android's back go back one level.
// Settings changed here are «in prova» (Session level): live, listed, saved for this game or its
// console, or undone; closing the game drops them.
//
// Controller / keyboard: opening the menu focuses the selected slot; B / Esc go back one level
// (App.vue sends Esc here as a «back» step), and from the main screen resume the game.
// Hints show only the input in use: Esc with mouse and keyboard, B with a controller, none on touch.
// Y (the `search` step) or Ctrl + K open the search everywhere over the menu (X01, #32,
// useMenuSearch.ts): the game's shortcuts run, a setting opens the page here that shows it.
//
// This component wires the state (quick-menu/use*.ts) to the parts (quick-menu/*.vue).
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import type { GameInfo, LibraryGame, ScreenLayoutId, ScreenStatus, Speed, VideoMode } from '@/emu'
import ScreenEditor from '@/components/screen-editor/ScreenEditor.vue'
import TouchEditor from '@/components/touch/TouchEditor.vue'
import { NO_INSET, type InsetPx } from '@/components/game/inset'

import { useNavScope } from '@/nav'
import { inputMode } from '@/lib/inputMode'
import { useErrorSink } from '@/composables/useToasts'
import { lockReason } from '@/lib/settingsModel'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import ControlsPage from './ControlsPage.vue'
import MenuFooter from './MenuFooter.vue'
import PhoneHome from './PhoneHome.vue'
import PhoneMore from './PhoneMore.vue'
import PhoneSaves from './PhoneSaves.vue'
import { MoreId, StripId, moreRows, stripButtons, type MoreRow } from './phoneMenu'
import { deviceEntries, useDeviceSettings } from './useDeviceSettings'
import { sameSpeed } from '@/lib/speeds'
import QuickMenuHome from './QuickMenuHome.vue'
import QuickSaves from './QuickSaves.vue'
import RestartConfirm from './RestartConfirm.vue'
import ScreenCardView from './ScreenCard.vue'
import ScreenPicks from './ScreenPicks.vue'
import ScreensBar from './ScreensBar.vue'
import ShortcutsPage from './ShortcutsPage.vue'
import SpeedPage from './SpeedPage.vue'
import SubPageHeader from './SubPageHeader.vue'
import TrialPanel from './TrialPanel.vue'
import UndoLoadBanner from './UndoLoadBanner.vue'
import { backKeyLabel, searchKeyLabels } from './hints'
import { BAR_PANEL, MENU_PANEL, MenuInset, SIDE_PANEL, sheetPanel } from './panel'
import {
  cycleStartingAt,
  layoutCard,
  settingCard,
  unavailableOptions,
  videoPathCard,
  type ScreenCard,
} from './screenCards'
import { pickChanges, screenPicks, type ScreenPickId } from './screenPicks'
import { slotCount } from '@/components/details/gameSaves'
import { loadedLine, mainSlotCount } from './slots'
import {
  SpeedChoiceId,
  chosenSpeed,
  followedSpeed,
  modeSpeeds,
  speedChoices,
  toggledSpeed,
} from './speedChoices'
import { ToolId, type Tool } from './tools'
import { useControllerShortcuts } from './useControllerShortcuts'
import { useMenuInset } from './useMenuInset'
import { useMenuSearch } from './useMenuSearch'

import { useQuickMenuNavigation } from './useQuickMenuNavigation'
import { useQuickSlots } from './useQuickSlots'
import { useSay } from '@/i18n/message'
import { useTrialSession } from './useTrialSession'
import { QuickMenuView, SESSION_VIEWS, SUB_PAGE_TITLE, isSubPage, phoneParent } from './view'

const props = defineProps<{
  game: GameInfo
  speed: Speed
  videoMode: VideoMode
  nativeAvailable: boolean
  /** Name of the controller in use, null with none. */
  controller: string | null
  /** The slot the controller shortcuts save to and load from: selected when the menu opens. */
  currentSlot: number
  /** A quick save was loaded in this game and can be undone (`saves.undo`): the slot, else null. */
  undoLoadSlot: number | null
  muted: boolean
  fullscreen: boolean
  /** The open game's screens (NDS, 3DS: swap, the lid, the layout card); null until known. */
  screens: ScreenStatus | null
}>()
const emit = defineEmits<{
  videoMode: [VideoMode]
  resume: []
  speed: [Speed]
  save: [number]
  /** The phone's «Rendi rapido»: this slot becomes the quick slot. */
  quickSlot: [number]
  /** The phone's «Riavvolgi» pressed (true) and let go (false). */
  rewind: [boolean]
  load: [number]
  undoLoad: []
  reset: []
  close: []
  mute: [boolean]
  fullscreen: []
  screenshot: []
  /** The edges the menu keeps clear of the game, CSS px: G09's bar on desktop, the phone's sheet
   * or side panel (none: the whole window); the game is fitted in the rest. */
  inset: [InsetPx]
  /** NDS / 3DS: «Scambia gli schermi», «Chiudi il coperchio», G09's layout card. */
  swapScreens: []
  toggleLid: []
  layout: [ScreenLayoutId]
  /** From the search (X01): the FPS shown or hidden, the next layout, another game opened. */
  fps: []
  nextLayout: []
  openGame: [LibraryGame]
}>()

const { t } = useI18n()
const say = useSay()
const failWith = useErrorSink()

// ---- State ----
const root = ref<HTMLElement | null>(null)
const {
  schema,
  rows: sessionRows,
  value: settingValue,
  set: setSession,
  commit,
  discard,
  title: trialTitle,
  entriesOf,
  start: startSession,
} = useTrialSession(failWith)
const { rows: deviceRows, load: loadDevice, set: setDevice } = useDeviceSettings(failWith)
const {
  slots,
  chosen,
  chosenEmpty,
  autoChosen,
  savedSlot,
  lastSaved,
  load: loadSlots,
  pick,
  save,
  loadChosen,
  undoSave,
} = useQuickSlots({
  count: () => slotCount(settingValue('saves.slots')),
  initial: props.currentSlot,
  onSave: slot => emit('save', slot),
  onLoad: slot => emit('load', slot),
  onError: failWith,
})
const { combos, modifier, boundCount, load: loadShortcuts } = useControllerShortcuts(failWith)
const { view, open, back, home, focusChosenSlot } = useQuickMenuNavigation(root, {
  resume: () => emit('resume'),
  home: () => void loadSlots(),
  parent: v => (phone.value ? phoneParent(v) : QuickMenuView.Home),
})

// ---- Layout: desktop panel, G09 bar, phone sheet, phone side panel ----
const layout = useDeviceLayout()
/** The phone, upright or not (its tools, its video methods). */
const phone = computed(() => layout.device.value === 'phone')
/** The phone held upright: the bottom sheet; on its side, the side panel (the same content). */
const sheet = layout.phone
/** G09 on desktop: the bar at the bottom instead of the panel. */
const barShown = computed(() => view.value === QuickMenuView.Screens && !phone.value)
/** C09: the screens editor over the whole game, in place of the menu. */
const editorShown = computed(() => view.value === QuickMenuView.ScreenEditor)
/** C06 (#29): the touch controls editor over the whole game, in place of the menu. */
const touchEditorShown = computed(() => view.value === QuickMenuView.TouchEditor)
/** The game shows through on G09 and in the editors (and beside the phone's sheet or panel). */
const dimmed = computed(
  () =>
    view.value !== QuickMenuView.Screens &&
    view.value !== QuickMenuView.ScreenEditor &&
    view.value !== QuickMenuView.TouchEditor &&
    !phone.value,
)
const panelClass = computed(() =>
  sheet.value ? sheetPanel(view.value) : phone.value ? SIDE_PANEL : MENU_PANEL,
)
/** What the menu keeps clear of the game: the phone's panel beside or under it (#29). */
const insetKind = computed<MenuInset>(() =>
  barShown.value || touchEditorShown.value
    ? MenuInset.Own
    : editorShown.value || !phone.value
      ? MenuInset.None
      : sheet.value
        ? MenuInset.Sheet
        : MenuInset.Side,
)
useMenuInset(
  root,
  () => insetKind.value,
  inset => emit('inset', inset),
)

const consoleName = computed(() => t(`console.${props.game.consoleId}`))
const backKey = computed(() => {
  const key = backKeyLabel(inputMode.value)
  return key ? t(key) : null
})
const pageTitle = computed(() => {
  if (!isSubPage(view.value)) return ''
  if (phone.value && view.value === QuickMenuView.Slots) return t('qm.savesPage')
  return t(SUB_PAGE_TITLE[view.value])
})
/** Where back leads from a page: Controlli from the shortcuts, else the menu; on the phone
 * «Indietro» (one level up). */
const backLabel = computed(() =>
  phone.value
    ? t('qm.back')
    : view.value === QuickMenuView.Shortcuts
      ? t('qm.row.controls.name')
      : t('qm.label'),
)

// ---- Quick saves ----
const shownSlots = computed(() =>
  view.value === QuickMenuView.Home
    ? slots.value.slice(0, mainSlotCount(phone.value))
    : slots.value,
)
/** A load that can be undone (`saves.undo` on): its slot, else null. */
const undoableLoad = computed(() =>
  settingValue('saves.undo') !== false ? props.undoLoadSlot : null,
)
/** With a controller, «Annulla il caricamento» sits at the top until the next save or load; on
 * the phone always (the toast told where to undo it). */
const showUndoLoad = computed(
  () => undoableLoad.value !== null && (phone.value || inputMode.value === 'controller'),
)
const loadedText = computed(() => say(loadedLine(props.undoLoadSlot ?? 0)))

// ---- The phone's first level and «Altro» (owner, 2026-11-22) ----
/** The quick slot: Rust's current slot, the controller shortcuts' («Salva / Carica rapido»). */
const quickInfo = computed(() => slots.value.find(s => s.slot === props.currentSlot))
const savedLine = computed(() =>
  savedSlot.value === null
    ? null
    : {
        slot: savedSlot.value,
        thumb: slots.value.find(s => s.slot === savedSlot.value)?.thumbUrl ?? null,
      },
)
const strip = computed(() =>
  stripButtons({
    rewind: settingValue('speed.rew') === true,
    fastForward: sameSpeed(props.speed, modes.value.ff),
    muted: props.muted,
  }),
)
function runStrip(id: StripId, e: Event) {
  switch (id) {
    case StripId.FastForward:
      return emit('speed', toggledSpeed(props.speed, modes.value.ff))
    case StripId.Mute:
      return emit('mute', !props.muted)
    case StripId.Close:
      // The close flow, as the desktop's «Chiudi il gioco» (MenuFooter).
      return emit('close')
    case StripId.More:
      return open(QuickMenuView.More, e)
    case StripId.Rewind:
      // Held: PhoneHome sends the press and the release.
      return
  }
}
const more = computed(() => moreRows({ dualScreen: dualScreen.value, screens: props.screens }))
const SPEED_NAME: Record<SpeedChoiceId, string> = {
  [SpeedChoiceId.Normal]: 'qm.normal',
  [SpeedChoiceId.FastForward]: 'qm.ff',
  [SpeedChoiceId.SlowMotion]: 'qm.slow',
}
const moreValues = computed<Partial<Record<MoreId, string>>>(() => {
  const speedId = chosenSpeed(choices.value, props.speed)
  return {
    [MoreId.Saves]: t('qm.quickSlotIs', { slot: props.currentSlot }),
    ...(speedId ? { [MoreId.Speed]: t(SPEED_NAME[speedId]) } : {}),
  }
})
function runMore(row: MoreRow, e: Event) {
  if (row.page) return open(row.page, e)
  switch (row.id) {
    case MoreId.Screenshot:
      return emit('screenshot')
    case MoreId.SwapScreens:
      return emit('swapScreens')
    case MoreId.CloseLid:
      return emit('toggleLid')
    case MoreId.Restart:
      return open(QuickMenuView.Restart, e)
    case MoreId.Saves:
    case MoreId.Speed:
    case MoreId.Screens:
    case MoreId.Controls:
    case MoreId.Layout:
      // Pages (and the C06 editor): opened above.
      return
  }
}

// ---- Speed (G07, G16) ----
const modes = computed(() => modeSpeeds(settingValue('speed.ffval'), settingValue('speed.slow')))
const choices = computed(() => speedChoices(modes.value.ff, modes.value.slow))
// G16 changed the speed of the mode in use: the game follows at once.
watch(modes, (now, before) => {
  const next = followedSpeed(props.speed, now, before)
  if (next) emit('speed', next)
})

// ---- Tools (listed by QuickMenuHome) ----
const dualScreen = computed(() => props.game.consoleId === 'nds' || props.game.consoleId === '3ds')
function runTool(tool: Tool, e: Event) {
  switch (tool.id) {
    case ToolId.Screenshot:
      return emit('screenshot')
    case ToolId.Mute:
      return emit('mute', !props.muted)
    case ToolId.Fullscreen:
      return emit('fullscreen')
    case ToolId.Restart:
      return open(QuickMenuView.Restart, e)
    case ToolId.SwapScreens:
      return emit('swapScreens')
    case ToolId.CloseLid:
      return emit('toggleLid')
  }
}
function restart() {
  home()
  emit('reset')
}

// ---- G09: the layout (NDS, 3DS), one card per setting, then the video method ----
/** The layout card's step: shown at once, and «in prova» as the layout the game opens with
 * (`screens.cycle` turned to start with it), saved or undone with the other changes. */
function stepLayout(id: ScreenLayoutId) {
  emit('layout', id)
  const cycle = settingValue('screens.cycle')
  if (Array.isArray(cycle)) void setSession('screens.cycle', cycleStartingAt(cycle, id))
}
const cards = computed<ScreenCard[]>(() => {
  const s = schema.value
  if (!s) return []
  const screens = props.screens
  return [
    ...(dualScreen.value && screens?.dualScreen
      ? [
          layoutCard({
            status: screens,
            row: sessionRows.value['screens.cycle'],
            consoleName: consoleName.value,
            set: stepLayout,
          }),
        ]
      : []),
    ...entriesOf(QuickMenuView.Screens).map(e =>
      settingCard({
        entry: e,
        row: sessionRows.value[e.key],
        value: settingValue(e.key),
        reason: lockReason(e, s, 'session'),
        unavailable: unavailableOptions(e.key, settingValue),
        consoleName: consoleName.value,
        set: id => void setSession(e.key, id),
      }),
    ),
    videoPathCard({
      mode: props.videoMode,
      nativeAvailable: props.nativeAvailable,
      phone: phone.value,
      set: mode => emit('videoMode', mode),
    }),
  ]
})

// G09 quick picks: several settings «in prova» in one press.
const picks = computed(() => {
  const s = schema.value
  if (!s) return []
  return screenPicks({
    value: settingValue,
    locked: key => {
      const e = s.entries.find(x => x.key === key)
      return !e || lockReason(e, s, 'session') !== null
    },
  })
})
async function applyPick(id: ScreenPickId) {
  for (const [key, v] of pickChanges(id, settingValue)) await setSession(key, v)
}

// ---- The search everywhere (X01, #32) ----
/** The settings the menu's pages show «in prova». */
const menuSettings = computed(() => SESSION_VIEWS.flatMap(v => entriesOf(v).map(e => e.key)))
const { openSearch } = useMenuSearch({
  root,
  context: () => ({
    playing: props.game.id,
    dualScreen: dualScreen.value,
    lidSupported: props.screens?.lidSupported ?? true,
    loadBlock: props.game.stateLoadBlock,
    slot: { chosen: chosen.value, empty: chosenEmpty.value, auto: autoChosen.value },
    settings: menuSettings.value,
  }),
  slots: {
    numbers: () => slots.value.map(s => s.slot),
    chosen: () => chosen.value,
    pick,
    save,
    loadChosen,
  },
  speed: () => props.speed,
  modes: () => modes.value,
  run: {
    resume: () => emit('resume'),
    speed: s => emit('speed', s),
    screenshot: () => emit('screenshot'),
    fps: () => emit('fps'),
    swapScreens: () => emit('swapScreens'),
    nextLayout: () => emit('nextLayout'),
    toggleLid: () => emit('toggleLid'),
    fullscreen: () => emit('fullscreen'),
    openGame: g => emit('openGame', g),
  },
  openPage: v => open(v, null),
})
useNavScope(root, { back, autofocus: true, search: openSearch })
const searchKeys = computed(() => searchKeyLabels(inputMode.value).map(k => t(k)))

onMounted(async () => {
  await startSession()
  await loadDevice()
  await loadSlots()
  void focusChosenSlot()
  await loadShortcuts()
})
onBeforeUnmount(() => emit('inset', NO_INSET))
</script>

<template>
  <div class="pointer-events-none fixed inset-0 z-menu *:pointer-events-auto">
    <!-- The paused game behind the menu: dimmed (blurred where the webview draws it). -->
    <div
      v-if="dimmed"
      class="absolute inset-0 bg-game-scrim backdrop-blur-veil"
      aria-hidden="true"
    ></div>
    <!-- The phone: a tap on the game beside or above the menu resumes it (there is no handle). -->
    <div
      v-if="phone && !editorShown && !touchEditorShown"
      class="absolute inset-0"
      aria-hidden="true"
      @click="emit('resume')"
    ></div>

    <section
      v-if="barShown"
      ref="root"
      :class="BAR_PANEL"
      role="dialog"
      :aria-label="t('qm.row.screens.name')"
    >
      <ScreensBar
        :cards="cards"
        :picks="picks"
        :trial-title="trialTitle"
        :hint="backKey"
        :dual-screen="dualScreen && !!screens?.dualScreen"
        @back="back"
        @discard="discard"
        @commit-game="commit('game')"
        @inset="px => emit('inset', px)"
        @pick="applyPick"
        @customise="e => open(QuickMenuView.ScreenEditor, e)"
      />
    </section>

    <!-- C09 over the game at its real size; the menu's scope stays its parent. -->
    <div v-else-if="editorShown" ref="root" class="contents">
      <ScreenEditor :console="game.consoleId" :scope="null" :status="screens" @close="back" />
    </div>

    <!-- C06 over the game, the same way. -->
    <div v-else-if="touchEditorShown" ref="root" class="contents">
      <TouchEditor :console="game.consoleId" live @close="back" @inset="px => emit('inset', px)" />
    </div>

    <section
      v-else
      ref="root"
      :class="panelClass"
      :role="view === QuickMenuView.Restart ? 'alertdialog' : 'dialog'"
      :aria-label="view === QuickMenuView.Restart ? t('qm.restartTitle') : t('qm.label')"
    >
      <RestartConfirm v-if="view === QuickMenuView.Restart" @cancel="back" @confirm="restart" />

      <template v-else>
        <header v-if="!phone && view === QuickMenuView.Home" class="flex items-baseline gap-8">
          <span class="text-heading">{{ game.title }}</span>
          <span class="text-caption text-ink-muted">{{ consoleName }}</span>
        </header>

        <UndoLoadBanner
          v-if="!phone && showUndoLoad && view === QuickMenuView.Home"
          :text="loadedText"
          @undo="emit('undoLoad')"
        />

        <SubPageHeader
          v-if="view !== QuickMenuView.Home"
          :title="pageTitle"
          :back-label="backLabel"
          :hint="backKey"
          :touch="phone"
          @back="back"
        />

        <!-- The middle scrolls when it does not fit (the phone on its side): the header and the
             footer stay; a swipe pans it and never reaches the game behind. Room on every side
             for the focus ring (outline and glow), which the scrolling box would clip (#131). -->
        <div
          class="-mx-8 -my-4 flex min-h-0 touch-pan-y flex-col gap-12 overflow-y-auto overscroll-contain px-8 py-4 touch:flex-1 touch:gap-8"
        >
          <!-- The phone's levels (owner, 2026-11-22). -->
          <PhoneHome
            v-if="phone && view === QuickMenuView.Home"
            :side="!sheet"
            :quick-slot="currentSlot"
            :quick-info="quickInfo"
            :load-block="game.stateLoadBlock"
            :strip="strip"
            :saved="savedLine"
            :undo-load-text="showUndoLoad ? loadedText : null"
            @quick-save="save(currentSlot)"
            @quick-load="emit('load', currentSlot)"
            @resume="emit('resume')"
            @undo-save="undoSave"
            @undo-load="emit('undoLoad')"
            @strip="runStrip"
            @rewind="on => emit('rewind', on)"
          />
          <PhoneMore
            v-else-if="phone && view === QuickMenuView.More"
            :rows="more"
            :values="moreValues"
            @run="runMore"
          />
          <PhoneSaves
            v-else-if="phone && view === QuickMenuView.Slots"
            :slots="slots"
            :chosen="chosen"
            :chosen-empty="chosenEmpty"
            :auto-chosen="autoChosen"
            :quick-slot="currentSlot"
            :saved-slot="savedSlot"
            :load-block="game.stateLoadBlock"
            :undo-load-slot="undoableLoad"
            :last-saved="lastSaved"
            @pick="pick"
            @load="loadChosen"
            @save="save()"
            @make-quick="emit('quickSlot', chosen)"
            @undo-save="undoSave"
            @undo-load="emit('undoLoad')"
          />

          <QuickSaves
            v-else-if="view === QuickMenuView.Home || view === QuickMenuView.Slots"
            :slots="shownSlots"
            :chosen="chosen"
            :chosen-empty="chosenEmpty"
            :auto-chosen="autoChosen"
            :saved-slot="savedSlot"
            :load-block="game.stateLoadBlock"
            :all="view === QuickMenuView.Slots"
            :touch="phone"
            @pick="pick"
            @more="e => open(QuickMenuView.Slots, e)"
            @load="loadChosen"
            @save="save"
            @undo-save="undoSave"
          />

          <QuickMenuHome
            v-if="!phone && view === QuickMenuView.Home"
            :choices="choices"
            :speed="speed"
            :ff-option="settingValue('speed.ffval')"
            :slow-option="settingValue('speed.slow')"
            :sheet="phone"
            :phone="phone"
            :dual-screen="dualScreen"
            :muted="muted"
            :fullscreen="fullscreen"
            :screens="screens"
            @speed="s => emit('speed', s)"
            @open="open"
            @run="runTool"
          />

          <SpeedPage
            v-else-if="view === QuickMenuView.Speed"
            :entries="entriesOf(QuickMenuView.Speed)"
            :rows="sessionRows"
            :schema="schema"
            @set="setSession"
          />

          <!-- Phone G09: the picks and the cards in the sheet, the game at full brightness above. -->
          <template v-else-if="view === QuickMenuView.Screens">
            <ScreenPicks :picks="picks" @pick="applyPick" />
            <div class="grid grid-cols-[1fr_1fr] gap-8">
              <ScreenCardView v-for="c in cards" :key="c.key" :card="c" />
            </div>
          </template>

          <ControlsPage
            v-else-if="view === QuickMenuView.Controls"
            :controller="controller"
            :entries="entriesOf(QuickMenuView.Controls)"
            :rows="sessionRows"
            :schema="schema"
            :modifier="modifier"
            :shortcut-count="boundCount"
            :touch="layout.coarse"
            :device-entries="deviceEntries(QuickMenuView.Controls, schema)"
            :device-rows="deviceRows"
            @set="setSession"
            @set-device="setDevice"
            @shortcuts="e => open(QuickMenuView.Shortcuts, e)"
            @touch-edit="e => open(QuickMenuView.TouchEditor, e)"
          />

          <ShortcutsPage
            v-else-if="view === QuickMenuView.Shortcuts"
            :combos="combos"
            :modifier="modifier"
          />
        </div>

        <!-- Changes «in prova» (G16, G10 and the phone's G09). -->
        <TrialPanel
          v-if="trialTitle"
          :title="trialTitle"
          :console-name="consoleName"
          @discard="discard"
          @commit="commit"
        />

        <MenuFooter
          v-if="!phone && view === QuickMenuView.Home"
          :hint="backKey"
          :search-keys="searchKeys"
          :touch="phone"
          @close="emit('close')"
          @resume="emit('resume')"
        />
      </template>
    </section>
  </div>
</template>
