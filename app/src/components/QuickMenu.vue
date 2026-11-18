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
import { Gamepad2, Rows2 } from '@lucide/vue'
import type { GameInfo, Speed, VideoMode } from '../emu'
import { useNavScope } from '../nav'
import { inputMode } from '../inputMode'
import { lockReason } from '../settingsModel'
import ControlsPage from './quick-menu/ControlsPage.vue'
import MenuFooter from './quick-menu/MenuFooter.vue'
import PageRow from './quick-menu/PageRow.vue'
import PendingValues from './quick-menu/PendingValues.vue'
import PhoneGrid from './quick-menu/PhoneGrid.vue'
import QuickSaves from './quick-menu/QuickSaves.vue'
import RestartConfirm from './quick-menu/RestartConfirm.vue'
import ScreenCardView from './quick-menu/ScreenCard.vue'
import ScreensBar from './quick-menu/ScreensBar.vue'
import ShortcutsPage from './quick-menu/ShortcutsPage.vue'
import SpeedPage from './quick-menu/SpeedPage.vue'
import SpeedPicker from './quick-menu/SpeedPicker.vue'
import SubPageHeader from './quick-menu/SubPageHeader.vue'
import ToolTiles from './quick-menu/ToolTiles.vue'
import TrialPanel from './quick-menu/TrialPanel.vue'
import UndoLoadBanner from './quick-menu/UndoLoadBanner.vue'
import { backKeyLabel } from './quick-menu/hints'
import { BAR_PANEL, MENU_PANEL, sheetPanel } from './quick-menu/panel'
import { settingCard, videoPathCard, type ScreenCard } from './quick-menu/screenCards'
import { loadedLine, mainSlotCount, slotCount } from './quick-menu/slots'
import { followedSpeed, modeSpeeds, speedChoices } from './quick-menu/speedChoices'
import { ToolId, quickTools, toolsNotYet, type Tool } from './quick-menu/tools'
import { useControllerShortcuts } from './quick-menu/useControllerShortcuts'
import { usePortrait } from './quick-menu/usePortrait'
import { useQuickMenuNavigation } from './quick-menu/useQuickMenuNavigation'
import { useQuickSlots } from './quick-menu/useQuickSlots'
import { useSay } from './quick-menu/useSay'
import { useTrialSession } from './quick-menu/useTrialSession'
import { QuickMenuView, SUB_PAGE_TITLE, isSubPage } from './quick-menu/view'

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
  error: [string]
}>()

const { t } = useI18n()
const say = useSay()
const reportError = (text: string) => emit('error', text)

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
  follow: followSession,
} = useTrialSession(reportError)
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
  follow: followSaves,
} = useQuickSlots({
  count: () => slotCount(settingValue('saves.slots')),
  initial: props.currentSlot,
  onSave: slot => emit('save', slot),
  onLoad: slot => emit('load', slot),
  onError: reportError,
})
const { combos, modifier, boundCount, load: loadShortcuts } = useControllerShortcuts(reportError)
const { view, open, back, home, focusChosenSlot } = useQuickMenuNavigation(root, {
  resume: () => emit('resume'),
  home: () => void loadSlots(),
})
useNavScope(root, { back, autofocus: true })

// ---- Layout: desktop panel, G09 bar, phone sheet ----
const portrait = usePortrait()
const phone = computed(() => schema.value?.device === 'phone')
const sheet = computed(() => phone.value && portrait.value)
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

// ---- Tools ----
const dualScreen = computed(() => props.game.consoleId === 'nds' || props.game.consoleId === '3ds')
const tools = computed(() =>
  quickTools({
    phone: phone.value,
    dualScreen: dualScreen.value,
    muted: props.muted,
    fullscreen: props.fullscreen,
  }),
)
const notYet = computed(() => {
  const names = toolsNotYet(tools.value)
    .map(key => t(key))
    .join(', ')
  return names ? t('qm.toolsNotYet', { names }) : ''
})
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
        locked: lockReason(e, s, 'session') !== null,
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

onMounted(async () => {
  await startSession()
  await loadSlots()
  void focusChosenSlot()
  await loadShortcuts()
  await followSession()
  await followSaves()
})
onBeforeUnmount(() => emit('inset', 0))
</script>

<template>
  <PendingValues>
    <div class="pointer-events-none fixed inset-0 z-menu *:pointer-events-auto">
      <!-- The paused game behind the menu: dimmed (blurred where the webview draws it). -->
      <div
        v-if="dimmed"
        class="absolute inset-0 bg-(--qm-scrim) backdrop-blur-[2px]"
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
          :trial-title="trialTitle"
          :hint="backKey"
          @back="back"
          @discard="discard"
          @commit-game="commit('game')"
          @inset="px => emit('inset', px)"
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
          class="h-4 w-[40px] shrink-0 self-center rounded-full bg-(--qm-grip)"
          aria-hidden="true"
        ></span>

        <RestartConfirm
          v-if="view === QuickMenuView.Restart"
          :touch="sheet"
          @cancel="back"
          @confirm="restart"
        />

        <template v-else>
          <header v-if="!sheet && view === QuickMenuView.Home" class="flex items-baseline gap-8">
            <span class="text-heading">{{ game.title }}</span>
            <span class="text-caption text-ink-muted">{{ consoleName }}</span>
          </header>

          <UndoLoadBanner
            v-if="showUndoLoad && view === QuickMenuView.Home"
            :text="loadedText"
            :touch="sheet"
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

          <div
            class="flex min-h-0 flex-col overflow-y-auto"
            :class="sheet ? 'flex-1 gap-10' : 'gap-16'"
          >
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

            <template v-if="view === QuickMenuView.Home">
              <SpeedPicker
                :choices="choices"
                :speed="speed"
                :ff-option="settingValue('speed.ffval')"
                :slow-option="settingValue('speed.slow')"
                :touch="sheet"
                @speed="s => emit('speed', s)"
                @set-speeds="e => open(QuickMenuView.Speed, e)"
              />
              <template v-if="!sheet">
                <ToolTiles
                  :tools="tools"
                  :three-columns="dualScreen"
                  :not-yet="notYet"
                  @run="runTool"
                />
                <nav class="grid grid-cols-[1fr_1fr] gap-8" :aria-label="t('qm.moreLabel')">
                  <PageRow
                    :icon="Rows2"
                    :label="t('qm.row.screens.name')"
                    @click="e => open(QuickMenuView.Screens, e)"
                  />
                  <PageRow
                    :icon="Gamepad2"
                    :label="t('qm.row.controls.name')"
                    @click="e => open(QuickMenuView.Controls, e)"
                  />
                </nav>
              </template>
              <PhoneGrid v-else :tools="tools" :not-yet="notYet" @open="open" @run="runTool" />
            </template>

            <SpeedPage
              v-else-if="view === QuickMenuView.Speed"
              :entries="entriesOf(QuickMenuView.Speed)"
              :rows="sessionRows"
              :schema="schema"
              @set="setSession"
              @error="reportError"
            />

            <!-- Phone G09: the cards in the sheet, the game at full brightness above. -->
            <div v-else-if="view === QuickMenuView.Screens" class="grid grid-cols-[1fr_1fr] gap-8">
              <ScreenCardView v-for="c in cards" :key="c.key" :card="c" touch />
            </div>

            <ControlsPage
              v-else-if="view === QuickMenuView.Controls"
              :controller="controller"
              :entries="entriesOf(QuickMenuView.Controls)"
              :rows="sessionRows"
              :schema="schema"
              :modifier="modifier"
              :shortcut-count="boundCount"
              @set="setSession"
              @error="reportError"
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
            :touch="sheet"
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
  </PendingValues>
</template>
