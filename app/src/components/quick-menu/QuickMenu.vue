<script setup lang="ts">
// The quick menu (boards «MenuRapido3» G07–G11, G16, G17 and «Tel-MenuRapido3», 18/11).
// Desktop: a glass panel in the centre, 640 px wide and as tall as its content; the game is
// paused behind it, dimmed. Top to bottom: quick saves (Automatico + 1–4, «Altri slot e
// salvataggi» = G08), speed (Normale / Avanti veloce / Rallentatore, «Imposta le velocità» = G16),
// tools, the rows «Schermi e immagine» (G09: the menu becomes a bar at the bottom and the game
// stays visible above it) and «Controlli» (G10), then «Chiudi il gioco» and «Riprendi il gioco».
// Phone in portrait: a bottom sheet under the game, same order for the thumb.
// Settings changed here are «in prova» (Session level): live, listed, saved for this game or its
// console, or undone; closing the game drops them.
//
// Controller / keyboard: opening the menu focuses the selected slot; B / Esc go back one level
// (App.vue sends Esc here as a «back» step), and from the main screen resume the game.
// Hints show only the input in use: Esc with mouse and keyboard, B with a controller, none on touch.
//
// This component wires the state (quick-menu/use*.ts) to the parts (quick-menu/*.vue).
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import type { GameInfo, Speed, VideoMode } from '@/emu'
import { useNavScope } from '@/nav'
import { inputMode } from '@/lib/inputMode'
import { useErrorSink } from '@/composables/useToasts'
import { lockReason } from '@/lib/settingsModel'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import ControlsPage from './ControlsPage.vue'
import MenuFooter from './MenuFooter.vue'
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
import { backKeyLabel } from './hints'
import { BAR_PANEL, MENU_PANEL, sheetPanel } from './panel'
import { settingCard, unavailableOptions, videoPathCard, type ScreenCard } from './screenCards'
import { pickChanges, screenPicks, type ScreenPickId } from './screenPicks'
import { slotCount } from '@/components/details/gameSaves'
import { loadedLine, mainSlotCount } from './slots'
import { followedSpeed, modeSpeeds, speedChoices } from './speedChoices'
import { ToolId, type Tool } from './tools'
import { useControllerShortcuts } from './useControllerShortcuts'

import { useQuickMenuNavigation } from './useQuickMenuNavigation'
import { useQuickSlots } from './useQuickSlots'
import { useSay } from '@/i18n/message'
import { useTrialSession } from './useTrialSession'
import { QuickMenuView, SUB_PAGE_TITLE, isSubPage } from './view'

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
}>()
const emit = defineEmits<{
  videoMode: [VideoMode]
  resume: []
  speed: [Speed]
  save: [number]
  load: [number]
  undoLoad: []
  reset: []
  close: []
  mute: [boolean]
  fullscreen: []
  screenshot: []
  /** G09: px at the bottom of the window the bar covers (0 = none); the game shrinks above. */
  inset: [number]
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
const {
  slots,
  chosen,
  chosenEmpty,
  autoChosen,
  savedSlot,
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
})
useNavScope(root, { back, autofocus: true })

// ---- Layout: desktop panel, G09 bar, phone sheet ----
const layout = useDeviceLayout()
/** The phone, upright or not (its tools, its video methods). */
const phone = computed(() => layout.device.value === 'phone')
/** The phone held upright: the bottom sheet. */
const sheet = layout.phone
/** G09 on desktop: the bar at the bottom instead of the panel. */
const barShown = computed(() => view.value === QuickMenuView.Screens && !sheet.value)
/** The game shows through on G09 (and the phone's sheet sits under it): no dimming. */
const dimmed = computed(() => view.value !== QuickMenuView.Screens && !sheet.value)
const panelClass = computed(() => (sheet.value ? sheetPanel(view.value) : MENU_PANEL))

const consoleName = computed(() => t(`console.${props.game.consoleId}`))
const backKey = computed(() => {
  const key = backKeyLabel(inputMode.value)
  return key ? t(key) : null
})
const pageTitle = computed(() => (isSubPage(view.value) ? t(SUB_PAGE_TITLE[view.value]) : ''))
/** Where back leads from a page: Controlli from the shortcuts, else the menu. */
const backLabel = computed(() =>
  !sheet.value && view.value === QuickMenuView.Shortcuts
    ? t('qm.row.controls.name')
    : t('qm.label'),
)

// ---- Quick saves ----
const shownSlots = computed(() =>
  view.value === QuickMenuView.Home
    ? slots.value.slice(0, mainSlotCount(sheet.value))
    : slots.value,
)
/** With a controller, «Annulla il caricamento» sits at the top until the next save or load. */
const showUndoLoad = computed(
  () =>
    props.undoLoadSlot !== null &&
    settingValue('saves.undo') !== false &&
    inputMode.value === 'controller',
)
const loadedText = computed(() => say(loadedLine(props.undoLoadSlot ?? 0)))

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
    case ToolId.CloseLid:
      // No backend yet: the tile is disabled.
      return
  }
}
function restart() {
  home()
  emit('reset')
}

// ---- G09: one card per setting, then the video method ----
const cards = computed<ScreenCard[]>(() => {
  const s = schema.value
  if (!s) return []
  return [
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

onMounted(async () => {
  await startSession()
  await loadSlots()
  void focusChosenSlot()
  await loadShortcuts()
})
onBeforeUnmount(() => emit('inset', 0))
</script>

<template>
  <div class="pointer-events-none fixed inset-0 z-menu *:pointer-events-auto">
    <!-- The paused game behind the menu: dimmed (blurred where the webview draws it). -->
    <div
      v-if="dimmed"
      class="absolute inset-0 bg-game-scrim backdrop-blur-veil"
      aria-hidden="true"
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
        @back="back"
        @discard="discard"
        @commit-game="commit('game')"
        @inset="px => emit('inset', px)"
        @pick="applyPick"
      />
    </section>

    <section
      v-else
      ref="root"
      :class="panelClass"
      :role="view === QuickMenuView.Restart ? 'alertdialog' : 'dialog'"
      :aria-label="view === QuickMenuView.Restart ? t('qm.restartTitle') : t('qm.label')"
    >
      <span
        v-if="sheet"
        class="h-4 w-grip shrink-0 self-center rounded-full bg-grip"
        aria-hidden="true"
      ></span>

      <RestartConfirm v-if="view === QuickMenuView.Restart" @cancel="back" @confirm="restart" />

      <template v-else>
        <header v-if="!sheet && view === QuickMenuView.Home" class="flex items-baseline gap-8">
          <span class="text-heading">{{ game.title }}</span>
          <span class="text-caption text-ink-muted">{{ consoleName }}</span>
        </header>

        <UndoLoadBanner
          v-if="showUndoLoad && view === QuickMenuView.Home"
          :text="loadedText"
          @undo="emit('undoLoad')"
        />

        <SubPageHeader
          v-if="view !== QuickMenuView.Home"
          :title="pageTitle"
          :back-label="backLabel"
          :hint="backKey"
          :touch="sheet"
          @back="back"
        />

        <div class="flex min-h-0 flex-col gap-16 overflow-y-auto touch:flex-1 touch:gap-10">
          <QuickSaves
            v-if="view === QuickMenuView.Home || view === QuickMenuView.Slots"
            :slots="shownSlots"
            :chosen="chosen"
            :chosen-empty="chosenEmpty"
            :auto-chosen="autoChosen"
            :saved-slot="savedSlot"
            :all="view === QuickMenuView.Slots"
            :touch="sheet"
            @pick="pick"
            @more="e => open(QuickMenuView.Slots, e)"
            @load="loadChosen"
            @save="save"
            @undo-save="undoSave"
          />

          <QuickMenuHome
            v-if="view === QuickMenuView.Home"
            :choices="choices"
            :speed="speed"
            :ff-option="settingValue('speed.ffval')"
            :slow-option="settingValue('speed.slow')"
            :sheet="sheet"
            :phone="phone"
            :dual-screen="dualScreen"
            :muted="muted"
            :fullscreen="fullscreen"
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
            @set="setSession"
            @shortcuts="e => open(QuickMenuView.Shortcuts, e)"
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
          v-if="view === QuickMenuView.Home"
          :hint="backKey"
          :touch="sheet"
          @close="emit('close')"
          @resume="emit('resume')"
        />
      </template>
    </section>
  </div>
</template>
