<script setup lang="ts">
// The root: the first run, the shell, or the game being played, with the game flow's overlays
// and the toasts over all of them. The logic lives in the composables; this file composes.
import { onMounted, ref } from 'vue'
import { flow } from '@/components/game/gameFlow'
import type { Section } from '@/components/shell/sections'
import { useDensity } from '@/composables/useDensity'
import { coarsePointer, useDeviceLayout } from '@/composables/useDeviceLayout'
import { useClosing } from '@/components/closing/useClosing'
import { useDropTarget } from '@/components/library/useDropTarget'
import { useFirstRun } from '@/components/first-run/useFirstRun'
import { useGameKeys } from '@/components/game/useGameKeys'
import { useGameLaunch } from '@/components/game/useGameLaunch'
import { useGameSession } from '@/components/game/useGameSession'
import { useScreenTouch } from '@/components/game/useScreenTouch'
import { useShellEvents } from '@/components/shell/useShellEvents'
import { useToasts } from '@/composables/useToasts'
import { setErrorSink, settle } from '@/lib/errors'
import { useVideoMode } from '@/components/game/useVideoMode'
import AppShell from '@/components/shell/AppShell.vue'
import ClosingOverlay from '@/components/closing/ClosingOverlay.vue'
import FirstRun from '@/components/first-run/FirstRun.vue'
import GameCanvas from '@/components/game/GameCanvas.vue'
import GameFlowOverlays from '@/components/game/GameFlowOverlays.vue'
import GameSurround from '@/components/game/GameSurround.vue'
import LibraryDropTarget from '@/components/library/LibraryDropTarget.vue'
import QuickMenu from '@/components/quick-menu/QuickMenu.vue'
import ToastStack from '@/components/toasts/ToastStack.vue'
import TouchControls from '@/components/game/TouchControls.vue'
import GameHud from '@/components/game/GameHud.vue'

const toast = useToasts()
const { firstRun, decide: decideFirstRun, finish: finishFirstRun } = useFirstRun(toast)
const { videoMode, nativeAvailable, load: loadVideoMode, setVideoMode } = useVideoMode(toast)
const session = useGameSession(toast)
const { game, menuOpen, paused, stats, showFps, speed, settings, menuInset } = session
const launch = useGameLaunch(session, toast)
const keys = useGameKeys(session)
const drop = useDropTarget(game, toast)
const events = useShellEvents(session, toast, drop.listen)
const closing = useClosing(toast)

// The shell's section: kept while a game is open, so closing it comes back here.
const section = ref<Section>('library')
// Touch screen (Android): on-screen buttons while playing.
const touch = coarsePointer()
// DS games: the mouse or a finger on the bottom screen is the stylus (no menu open).
useScreenTouch(
  () => game.value?.consoleId === 'nds' && !menuOpen.value,
  videoMode,
  () => menuInset.value,
)
// The phone's layout (a phone held upright) sets the touch density for the whole interface.
const layout = useDeviceLayout()
useDensity(layout.phone)

// Startup: the keys and the backend's events first, so the controller works whatever fails
// after; each step on its own, a failed one reported (lib/errors.ts) and the next still run.
// The game given at startup waits for the video mode and the events it needs.
onMounted(async () => {
  setErrorSink(toast.failWith)
  keys.listen()
  session.watchStats()
  await Promise.all([
    settle(events.listen()),
    settle(closing.listen()),
    settle(decideFirstRun()),
    settle(loadVideoMode()),
    settle(session.loadDeviceState()),
  ])
  await settle(launch.openStartupGame())
})
</script>

<template>
  <!-- No game, first start on this device: the first run. -->
  <FirstRun v-if="!game && firstRun" @done="finishFirstRun" @error="toast.fail" />

  <!-- No game: the shell (rail, sections). A game that cannot start is G02's dialog. -->
  <AppShell
    v-else-if="!game && firstRun === false"
    v-model="section"
    @open-file="launch.openFile()"
    @open-game="launch.openLibraryGame"
    @error="toast.fail"
    @notice="text => toast.notify(text)"
  />

  <!-- In game: no interface, except discreet indicators and the quick menu. -->
  <template v-else-if="game">
    <GameSurround
      :mode="videoMode"
      :game-size="session.gameSize.value"
      :inset-bottom="menuInset"
      :game="session.playingGame.value"
    />
    <GameCanvas
      v-if="videoMode === 'webview'"
      :inset-bottom="menuInset"
      :colour-blind="settings.colourBlind"
      surround
    />
    <GameHud
      :speed="speed"
      :turbo-held="session.turboHeld.value"
      :ff-speed="settings.ffSpeed"
      :paused="paused && !menuOpen"
      :fps="showFps && stats ? stats.emuFps : null"
    />
    <TouchControls
      v-if="touch && !menuOpen"
      :shoulders="game.consoleId !== 'gb' && game.consoleId !== 'gbc'"
      @menu="session.setMenu(true)"
    />
    <QuickMenu
      v-if="menuOpen && !flow.ending"
      :game="game"
      :speed="speed"
      :video-mode="videoMode"
      :native-available="nativeAvailable"
      :controller="session.controller.value"
      :current-slot="session.comboSlot.value"
      :undo-load-slot="settings.undoAllowed ? session.undoLoadSlot.value : null"
      :muted="session.muted.value"
      :fullscreen="session.fullscreen.value"
      @video-mode="setVideoMode"
      @resume="session.setMenu(false)"
      @speed="session.setSpeed"
      @save="session.save"
      @load="session.loadFromMenu"
      @undo-load="session.undoLoad"
      @reset="session.restart"
      @close="session.close"
      @mute="session.setMuted"
      @fullscreen="session.toggleFullscreen"
      @screenshot="session.screenshot"
      @inset="session.setInset"
      @error="toast.fail"
    />
  </template>

  <LibraryDropTarget v-if="drop.dragOver.value && !game" />

  <GameFlowOverlays
    @start-error-close="flow.startError = null"
    @resume="launch.answerResume"
    @crash-copy="launch.crashCopy"
    @crash-library="launch.crashToLibrary"
    @crash-resume="launch.crashResume"
    @slow-close="flow.slow = null"
    @broken-use="launch.useLatestBackup"
    @broken-play="launch.playWithoutSave"
    @broken-close="launch.closeBroken"
  />

  <!-- Closing with a game open: saving, saved, or the question when it does not end well. -->
  <ClosingOverlay
    :view="closing.view.value"
    @wait="closing.wait"
    @close-anyway="closing.closeAnyway"
  />

  <ToastStack ref="toasts" :in-game="!!game" />
</template>
