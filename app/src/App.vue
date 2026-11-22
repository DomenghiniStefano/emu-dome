<script setup lang="ts">
// The root: the first run, the shell, or the game being played, with the game flow's overlays
// and the toasts over all of them. The logic lives in the composables; this file composes.
import { nextTick, onMounted, ref } from 'vue'
import { flow } from '@/components/game/gameFlow'
import { START, type Route } from '@/components/shell/route'
import { useDensity } from '@/composables/useDensity'
import { useUiMode } from '@/composables/useUiMode'
import { coarsePointer, useDeviceLayout } from '@/composables/useDeviceLayout'
import { useClosing } from '@/components/closing/useClosing'
import { useCloudEvents } from '@/components/cloud/useCloudEvents'
import { useDropTarget } from '@/components/library/useDropTarget'
import { useFirstRun } from '@/components/first-run/useFirstRun'
import { useGameKeys } from '@/components/game/useGameKeys'
import { useGameLaunch } from '@/components/game/useGameLaunch'
import { useGameSession } from '@/components/game/useGameSession'
import { useScreenStatus } from '@/components/game/useScreenStatus'
import { useScreenTouch } from '@/components/game/useScreenTouch'
import { useShellEvents } from '@/components/shell/useShellEvents'
import { createToasts, provideToasts } from '@/composables/useToasts'
import { createCoverRequest } from '@/components/cover/useCoverRequest'
import CoverCropDialog from '@/components/cover/CoverCropDialog.vue'
import { setErrorSink, settle } from '@/lib/errors'
import { hideSplash } from '@/lib/splash'
import { useVideoMode } from '@/components/game/useVideoMode'
import AppShell from '@/components/shell/AppShell.vue'
import ClosingOverlay from '@/components/closing/ClosingOverlay.vue'
import CloudFlowOverlays from '@/components/cloud/CloudFlowOverlays.vue'
import FirstRun from '@/components/first-run/FirstRun.vue'
import GameCanvas from '@/components/game/GameCanvas.vue'
import GameFlowOverlays from '@/components/game/GameFlowOverlays.vue'
import GameSurround from '@/components/game/GameSurround.vue'
import LibraryDropTarget from '@/components/library/LibraryDropTarget.vue'
import QuickMenu from '@/components/quick-menu/QuickMenu.vue'
import ToastStack from '@/components/toasts/ToastStack.vue'
import TouchControls from '@/components/game/TouchControls.vue'
import GameHud from '@/components/game/GameHud.vue'
import LidClosedCard from '@/components/game/LidClosedCard.vue'

// The toasts: provided to every screen (useToasts / useErrorSink) and the sink of lib/errors.ts
// from the start, so a failure before the stack is mounted waits in the list instead of being lost.
const toast = createToasts()
provideToasts(toast)
setErrorSink(toast.failWith)
const { firstRun, decide: decideFirstRun, finish: finishFirstRun } = useFirstRun(toast)
const { videoMode, nativeAvailable, load: loadVideoMode, setVideoMode } = useVideoMode(toast)
// «Usa come copertina» on the screenshot toast (#85): kept until the quick menu is open or the
// game has closed, so «Adatta la copertina» never covers the game being played.
const coverRequest = createCoverRequest()
const session = useGameSession(toast, coverRequest.ask)
const { game, menuOpen, paused, stats, showFps, speed, settings, menuInset } = session
const launch = useGameLaunch(session, toast)
const keys = useGameKeys(session)
const drop = useDropTarget(game, toast)
const events = useShellEvents(session, toast, drop.listen)
const closing = useClosing(toast)
// Google Drive sync (#31): the status, the transfers, G04 and G05 (CloudFlowOverlays).
const cloudEvents = useCloudEvents(session)
// NDS / 3DS: the layout shown, swapped, the lid (quick menu, G15).
const screens = useScreenStatus(toast)

// Where the shell is (components/shell/route.ts): kept while a game is open, so closing it comes
// back here (a game opened from its details comes back to them).
const route = ref<Route>(START)
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
// TV mode (#30): `general.ui`, or by itself full screen with a controller connected.
useUiMode(session.fullscreen, layout.device)

// Startup: the keys and the backend's events first, so the controller works whatever fails
// after; each step on its own, a failed one reported (lib/errors.ts) and the next still run.
// The game given at startup waits for the video mode and the events it needs. The startup splash
// (index.html, #101) fades out once the first run or the shell is decided and rendered, before
// the startup game, which shows its own starting scene.
onMounted(async () => {
  keys.listen()
  session.watchStats()
  await Promise.all([
    settle(events.listen()),
    settle(closing.listen()),
    settle(cloudEvents.listen()),
    settle(screens.listen()),
    settle(decideFirstRun()),
    settle(loadVideoMode()),
    settle(session.loadDeviceState()),
  ])
  await nextTick()
  hideSplash()
  await settle(launch.openStartupGame())
})
</script>

<template>
  <!-- No game, first start on this device: the first run. -->
  <FirstRun v-if="!game && firstRun" @done="finishFirstRun" />

  <!-- No game: the shell (rail, sections). A game that cannot start is G02's dialog. -->
  <AppShell
    v-else-if="!game && firstRun === false"
    v-model="route"
    @open-file="launch.openFile()"
    @open-game="launch.openLibraryGame"
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
    <LidClosedCard v-if="screens.status.value?.lidClosed && !menuOpen" @open="screens.toggleLid" />
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
      :screens="screens.status.value"
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
      @swap-screens="screens.swap"
      @toggle-lid="screens.toggleLid"
      @layout="screens.setLayout"
    />
  </template>

  <LibraryDropTarget v-if="drop.dragOver.value && !game" />

  <!-- «Adatta la copertina» for a screenshot taken in game: over the quick menu, or after the
       game has closed. -->
  <CoverCropDialog
    v-if="coverRequest.pending.value && (game ? menuOpen && !flow.ending : !firstRun)"
    :key="coverRequest.pending.value.name"
    :request="coverRequest.pending.value"
    @close="coverRequest.clear"
  />

  <GameFlowOverlays
    :load-block="game?.stateLoadBlock ?? null"
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

  <!-- Google Drive sync (#31): G05, G04 outside the game, G14's strip after a game closed. -->
  <CloudFlowOverlays v-if="game || firstRun === false" :in-game="!!game" />

  <!-- Closing with a game open: saving, saved, or the question when it does not end well. -->
  <ClosingOverlay
    :view="closing.view.value"
    @wait="closing.wait"
    @close-anyway="closing.closeAnyway"
  />

  <ToastStack :in-game="!!game" />
</template>
