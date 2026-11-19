<script setup lang="ts">
// The root: the first run, the shell, or the game being played, with the game flow's overlays
// and the toasts over all of them. The logic lives in the composables; this file composes.
import { onMounted, ref } from 'vue'
import { flow } from './gameFlow'
import type { Section } from './sections'
import { useDropTarget } from './composables/useDropTarget'
import { useFirstRun } from './composables/useFirstRun'
import { useGameKeys } from './composables/useGameKeys'
import { useGameLaunch } from './composables/useGameLaunch'
import { useGameSession } from './composables/useGameSession'
import { useShellEvents } from './composables/useShellEvents'
import { useToasts } from './composables/useToasts'
import { useVideoMode } from './composables/useVideoMode'
import AppShell from './components/AppShell.vue'
import FirstRun from './components/FirstRun.vue'
import GameCanvas from './components/GameCanvas.vue'
import GameFlowOverlays from './components/GameFlowOverlays.vue'
import GameSurround from './components/GameSurround.vue'
import LibraryDropTarget from './components/LibraryDropTarget.vue'
import QuickMenu from './components/QuickMenu.vue'
import ToastStack from './components/ToastStack.vue'
import TouchControls from './components/TouchControls.vue'
import GameHud from './components/game/GameHud.vue'

const toast = useToasts()
const { firstRun, decide: decideFirstRun, finish: finishFirstRun } = useFirstRun(toast)
const { videoMode, nativeAvailable, load: loadVideoMode, setVideoMode } = useVideoMode(toast)
const session = useGameSession(toast)
const { game, menuOpen, paused, stats, showFps, speed, settings, menuInset } = session
const launch = useGameLaunch(session, toast)
const keys = useGameKeys(session)
const drop = useDropTarget(game, toast)
const events = useShellEvents(session, toast, drop.listen)

// The shell's section: kept while a game is open, so closing it comes back here.
const section = ref<Section>('library')
// Touch screen (Android): on-screen buttons while playing.
const touch = window.matchMedia('(pointer: coarse)').matches

onMounted(async () => {
  void decideFirstRun()
  await loadVideoMode()
  keys.listen()
  await session.loadDeviceState()
  await events.listen()
  await launch.openStartupGame()
  session.watchStats()
})
</script>

<template>
  <!-- No game, first start on this device: the first run. -->
  <FirstRun v-if="!game && firstRun" @done="finishFirstRun" @error="toast.fail" />

  <!-- No game: the shell (rail, sections). The launch error it can show is G02's dialog now. -->
  <AppShell
    v-else-if="!game && firstRun === false"
    v-model="section"
    :error="null"
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
      v-if="menuOpen"
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

  <ToastStack ref="toasts" :in-game="!!game" />
</template>
