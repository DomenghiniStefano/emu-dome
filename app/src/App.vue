<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { Library } from '@lucide/vue'
import {
  AUTO_SLOT,
  emu,
  KEYMAP,
  type GameInfo,
  type HotkeyEvent,
  type LibraryGame,
  type Speed,
  type Stats,
  type VideoMode,
} from './emu'
import { startLibrary } from './library'
import { flow, gameStarted as flowStarted, sample } from './gameFlow'
import { errorText, gamepadName, speedLabel } from './i18n/labels'
import { dispatchNav, keyToNav, showFocusRing } from './nav'
import type { Section } from './sections'
import AppShell from './components/AppShell.vue'
import GameCanvas from './components/GameCanvas.vue'
import QuickMenu from './components/QuickMenu.vue'
import ToastStack from './components/ToastStack.vue'
import GameFlowOverlays from './components/GameFlowOverlays.vue'
import TouchControls from './components/TouchControls.vue'

const { t, n } = useI18n()

const game = ref<GameInfo | null>(null)
// The shell's section: kept while a game is open, so closing it comes back here.
const section = ref<Section>('library')
const menuOpen = ref(false)
// Paused with the controller combo (L3 + A), menu closed.
const paused = ref(false)
const showFps = ref(false)
const stats = ref<Stats | null>(null)
const speed = ref<Speed>({ kind: 'factor', value: 1 })
const error = ref<string | null>(null)
const toasts = ref<InstanceType<typeof ToastStack> | null>(null)
const videoMode = ref<VideoMode>('native')
const nativeAvailable = ref(false)
// Touch screen (Android): on-screen buttons while playing.
const touch = window.matchMedia('(pointer: coarse)').matches

async function setVideoMode(mode: VideoMode) {
  try {
    await emu.setVideoMode(mode)
  } catch (e) {
    toasts.value?.push(errorText(e), 'error')
    return
  }
  videoMode.value = mode
  applyBackground()
}

// With no native video underneath, the webview has to paint the background.
function applyBackground() {
  document.documentElement.style.background =
    videoMode.value === 'native' ? 'transparent' : 'var(--bg)'
}

// While the menu is open, Rust sends the controllers to the menu (emu://nav), not to the game.
watch(menuOpen, open => void emu.setMenuOpen(open))

const turboHeld = ref(false)
let statsTimer: number | undefined
const unlisten: Array<() => void> = []

// The last game opened, to reopen it after the emulator stopped (E11).
type Opened = { kind: 'path'; path: string } | { kind: 'library'; game: LibraryGame }
let lastOpened: Opened | null = null
// True while closing on purpose: the core's `stopped` event is then expected.
let closingNow = false
// When the open game started («stai giocando da …») and the controller in use (quick menu).
const startedAt = ref(Date.now())
const controller = ref<string | null>(null)

async function gameStarted(info: GameInfo, resumeAuto = false) {
  game.value = info
  startedAt.value = Date.now()
  speed.value = { kind: 'factor', value: 1 }
  menuOpen.value = false
  paused.value = false
  flowStarted()
  if (resumeAuto) {
    await emu.loadState(AUTO_SLOT)
  } else if (info.resumable) {
    // `saves.resume` = ask: hold the game until the answer.
    await emu.setPaused(true)
    flow.resumeAsk = true
  }
}

/** G01 while the core loads, G02 if it cannot start. */
async function start(opened: Opened, title: string, colour: string | null, resumeAuto = false) {
  error.value = null
  flow.launching = { title, colour }
  try {
    const info =
      opened.kind === 'path'
        ? await emu.openGame(opened.path)
        : await emu.library.open(opened.game.id)
    lastOpened = opened
    await gameStarted(info, resumeAuto)
  } catch (e) {
    flow.startError = { title, reason: errorText(e) }
  } finally {
    flow.launching = null
  }
}

/** The file name without folder and extension, as the title of a game outside the library. */
function fileTitle(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? path
  return name.replace(/\.[^.]+$/, '')
}

async function openGame(given?: string) {
  const path =
    given ??
    (await open({
      multiple: false,
      filters: [{ name: t('library.dialogFilter'), extensions: ['gba', 'gb', 'gbc'] }],
    }))
  if (typeof path !== 'string') return
  await start({ kind: 'path', path }, fileTitle(path), null)
}

/** A game from the library (L01): same flow, and the backend counts its play time. */
async function openLibraryGame(g: LibraryGame) {
  await start({ kind: 'library', game: g }, g.title, g.colour)
}

async function onResumeAnswer(resume: boolean) {
  flow.resumeAsk = false
  if (resume) await emu.loadState(AUTO_SLOT)
  await emu.setPaused(false)
}

// ---- E11: the emulator stopped ----
function crashed() {
  if (!game.value || flow.crashed) return
  menuOpen.value = false
  const s = stats.value
  flow.crashed = {
    console: game.value.consoleId,
    at: s?.autoSavedAt ?? null,
    detail: [
      `${game.value.title} · ${game.value.core}`,
      `frames ${s?.frames ?? 0} · ${s ? s.emuFps.toFixed(1) : '0'} / ${s?.targetFps ?? 0} FPS`,
      new Date().toISOString(),
    ].join('\n'),
  }
}
async function crashToLibrary() {
  flow.crashed = null
  await emu.abandonGame()
  game.value = null
}
async function crashResume() {
  const opened = lastOpened
  const title = game.value?.title ?? ''
  flow.crashed = null
  await emu.abandonGame()
  game.value = null
  if (opened)
    await start(opened, title, opened.kind === 'library' ? opened.game.colour : null, true)
}
async function crashCopy() {
  try {
    await navigator.clipboard.writeText(flow.crashed?.detail ?? '')
    toasts.value?.push(t('flow.detailsCopied'))
  } catch (e) {
    toasts.value?.push(String(e), 'error')
  }
}

// Files or folders dragged over the window, outside the game: overlay, then into the library.
const dragOver = ref(false)
async function onDrop(paths: string[]) {
  dragOver.value = false
  if (game.value || paths.length === 0) return
  try {
    await emu.library.addPaths(paths)
  } catch (e) {
    toasts.value?.push(errorText(e), 'error')
  }
}

/** G14: auto quick save and in-game save, then back to the section you came from. */
async function closeGame() {
  if (!game.value) return
  flow.closing = { title: game.value.title, autosave: game.value.autosaveOnExit }
  menuOpen.value = false
  closingNow = true
  const shown = Date.now()
  try {
    await emu.closeGame()
  } finally {
    // Long enough to read, never longer than the work.
    await new Promise(r => window.setTimeout(r, Math.max(0, 400 - (Date.now() - shown))))
    closingNow = false
    flow.closing = null
    game.value = null
  }
}

async function setMenu(openMenu: boolean) {
  if (!game.value) return
  menuOpen.value = openMenu
  paused.value = false
  await emu.setPaused(openMenu)
}

async function setSpeed(s: Speed) {
  speed.value = s
  await emu.setSpeed(s)
}

/** What a controller combo changed (the combo itself already ran in Rust). */
function onHotkey(h: HotkeyEvent) {
  switch (h.kind) {
    case 'fps':
      showFps.value = !showFps.value
      return
    case 'slot':
      toasts.value?.push(t('toast.slot', { slot: h.slot }))
      return
    case 'speed':
      speed.value = h.speed
      return
    case 'turbo':
      turboHeld.value = h.held
      return
    case 'paused':
      paused.value = h.paused
  }
}

function onKey(e: KeyboardEvent, pressed: boolean) {
  if (pressed) showFocusRing()
  if (e.code === 'Escape' && pressed) return void setMenu(!menuOpen.value)
  // Menu or shell: arrows move the focus, Enter confirms.
  if (!game.value || menuOpen.value) {
    const nav = pressed ? keyToNav(e) : null
    if (nav) {
      e.preventDefault()
      dispatchNav(nav)
    }
    return
  }
  if (pressed && !e.repeat) {
    if (e.code === 'F1') return void (e.preventDefault(), emu.saveState(1))
    if (e.code === 'F4') return void (e.preventDefault(), emu.loadState(1))
    if (e.code === 'F3') return void (e.preventDefault(), (showFps.value = !showFps.value))
  }
  // Turbo: Tab held down.
  if (e.code === 'Tab') {
    e.preventDefault()
    if (e.repeat || pressed === turboHeld.value) return
    turboHeld.value = pressed
    void emu.setSpeed(pressed ? { kind: 'max' } : speed.value)
    return
  }
  const button = KEYMAP[e.code]
  if (button) {
    e.preventDefault()
    if (!e.repeat) void emu.input(button, pressed)
  }
}

const down = (e: KeyboardEvent) => onKey(e, true)
const up = (e: KeyboardEvent) => onKey(e, false)

onMounted(async () => {
  ;[videoMode.value, nativeAvailable.value] = await emu.videoMode()
  applyBackground()
  window.addEventListener('keydown', down)
  window.addEventListener('keyup', up)
  unlisten.push(
    await emu.onMenu(() => {
      showFocusRing()
      void setMenu(!menuOpen.value)
    }),
    await emu.onNav(dispatchNav),
    await emu.onHotkey(onHotkey),
    await emu.onGamepadConnected(p => {
      controller.value = gamepadName(p)
      toasts.value?.push(t('toast.gamepadConnected', { name: gamepadName(p) }))
    }),
    await emu.onGamepadDisconnected(p => {
      if (controller.value === gamepadName(p)) controller.value = null
      toasts.value?.push(t('toast.gamepadDisconnected', { name: gamepadName(p) }))
    }),
    await emu.library.onDragDrop(e => {
      if (e.type === 'over') dragOver.value = !game.value
      else if (e.type === 'leave') dragOver.value = false
      else void onDrop(e.paths)
    }),
    ...(await startLibrary()),
    await emu.onEvent(ev => {
      // Slot 0 is the auto quick save: it works silently.
      if (ev.type === 'stateSaved' && ev.slot !== AUTO_SLOT)
        toasts.value?.push(t('toast.stateSaved', { slot: ev.slot }))
      if (ev.type === 'stateLoaded' && ev.slot !== AUTO_SLOT)
        toasts.value?.push(t('toast.stateLoaded', { slot: ev.slot }))
      if (ev.type === 'error') toasts.value?.push(errorText(ev.error), 'error')
      if (ev.type === 'stopped' && !closingNow) crashed()
    }),
  )
  const startup = await emu.startupGame()
  if (startup) await openGame(startup)
  statsTimer = window.setInterval(async () => {
    if (!game.value) return
    const s = await emu.stats()
    stats.value = s
    const running =
      !paused.value && !menuOpen.value && !flow.resumeAsk && !flow.crashed && !flow.slow
    const seen = sample(s, running)
    if (seen === 'hang') crashed()
    else if (seen === 'slow') flow.slow = { fps: s.emuFps, target: s.targetFps }
  }, 500)
})

onUnmounted(() => {
  window.removeEventListener('keydown', down)
  window.removeEventListener('keyup', up)
  unlisten.forEach(u => u())
  window.clearInterval(statsTimer)
})
</script>

<template>
  <!-- No game: the shell (rail, sections). -->
  <AppShell
    v-if="!game"
    v-model="section"
    :error="error"
    @open-file="openGame()"
    @open-game="openLibraryGame"
    @error="text => toasts?.push(text, 'error')"
  />

  <!-- In game: no interface, except discreet indicators and the quick menu. -->
  <template v-else>
    <GameCanvas v-if="videoMode === 'webview'" />
    <div class="hud">
      <div
        v-if="turboHeld || speed.kind === 'max' || (speed.kind === 'factor' && speed.value !== 1)"
        class="chip num"
      >
        {{ speedLabel(turboHeld ? { kind: 'max' } : speed) }}
      </div>
      <div v-if="paused && !menuOpen" class="chip">{{ t('hud.paused') }}</div>
      <div v-if="showFps && stats" class="chip num">
        {{
          t('hud.fps', {
            fps: n(stats.emuFps, { minimumFractionDigits: 1, maximumFractionDigits: 1 }),
          })
        }}
      </div>
    </div>
    <TouchControls
      v-if="touch && !menuOpen"
      :shoulders="game.console !== 'GB' && game.console !== 'GBC'"
      @menu="setMenu(true)"
    />
    <QuickMenu
      v-if="menuOpen"
      :game="game"
      :speed="speed"
      :video-mode="videoMode"
      :native-available="nativeAvailable"
      :started-at="startedAt"
      :controller="controller"
      @video-mode="setVideoMode"
      @resume="setMenu(false)"
      @speed="setSpeed"
      @save="(slot: number) => emu.saveState(slot)"
      @load="(slot: number) => emu.loadState(slot)"
      @reset="emu.reset()"
      @close="closeGame"
      @error="text => toasts?.push(text, 'error')"
    />
  </template>

  <!-- Drag and drop over the window: games and folders go into the library. -->
  <div v-if="dragOver && !game" class="drop" aria-hidden="true">
    <div class="drop-box">
      <Library :size="44" :stroke-width="1.5" />
      <div class="drop-text">{{ t('library.drop') }}</div>
    </div>
  </div>

  <GameFlowOverlays
    @start-error-close="flow.startError = null"
    @resume="onResumeAnswer"
    @crash-copy="crashCopy"
    @crash-library="crashToLibrary"
    @crash-resume="crashResume"
    @slow-close="flow.slow = null"
  />

  <ToastStack ref="toasts" :in-game="!!game" />
</template>

<style scoped>
.drop {
  position: fixed;
  inset: 0;
  z-index: 30;
  padding: 16px;
  display: flex;
  background: var(--scrim);
  backdrop-filter: blur(8px);
  pointer-events: none;
}
.drop-box {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  border: 2px dashed var(--accent-text);
  border-radius: var(--radius-16);
  color: var(--accent-text);
}
.drop-text {
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
  color: var(--ink);
}
.hud {
  position: fixed;
  top: 16px;
  left: 16px;
  display: flex;
  gap: 8px;
  pointer-events: none;
}
.chip {
  height: 28px;
  padding: 0 10px;
  display: flex;
  align-items: center;
  border-radius: var(--radius-6);
  background: var(--glass-thick);
  border: 1px solid var(--line);
  font-weight: 600;
}
</style>
