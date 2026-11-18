<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { Library } from '@lucide/vue'
import {
  AUTO_SLOT,
  emu,
  type GameInfo,
  type HotkeyEvent,
  type LibraryGame,
  type Speed,
  type Stats,
  type VideoMode,
} from './emu'
import { startLibrary } from './library'
import { flow, gameStarted as flowStarted, sample } from './gameFlow'
import { errorText, gamepadName, isErrorCode, speedLabel } from './i18n/labels'
import { dispatchNav, keyToNav, showFocusRing } from './nav'
import { inputMode, usedController, watchInput } from './inputMode'
import { FF_DEFAULT, speedFromOption } from './speeds'
import { keymap, loadKeymap } from './keymap'
import { keyShortcut, type KeyAction } from './keyboard'
import { nativeAvailable, videoMode } from './videoMode'
import type { Section } from './sections'
import AppShell from './components/AppShell.vue'
import GameCanvas from './components/GameCanvas.vue'
import QuickMenu from './components/QuickMenu.vue'
import ToastStack from './components/ToastStack.vue'
import GameFlowOverlays from './components/GameFlowOverlays.vue'
import TouchControls from './components/TouchControls.vue'
import FirstRun from './components/FirstRun.vue'

const { t, n } = useI18n()

const game = ref<GameInfo | null>(null)
// First run (#35): null until Rust answered whether this device still needs it.
const firstRun = ref<boolean | null>(null)
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
// The keyboard's keys follow the open game's settings, or «Tutti i giochi» without one.
watch(
  () => game.value !== null,
  inGame => void loadKeymap(inGame),
)

const turboHeld = ref(false)
let statsTimer: number | undefined
const unlisten: Array<() => void> = []

// The last game opened, to reopen it after the emulator stopped (E11).
type Opened = { kind: 'path'; path: string } | { kind: 'library'; game: LibraryGame }
let lastOpened: Opened | null = null
// True while closing on purpose: the core's `stopped` event is then expected.
let closingNow = false
// The controller in use (quick menu).
const controller = ref<string | null>(null)
// The current slot (L3 + ← →, F2 / F3 move it; L3 + R1 / L1, F1 / F4 use it): the quick menu
// opens on it. Rust holds it (`current_slot`), shared by the controller and the keyboard.
const comboSlot = ref(1)
// The slot of the last load, until the next save or load: «Annulla il caricamento» (`saves.undo`).
const undoLoadSlot = ref<number | null>(null)
// A load asked from the quick menu: its `stateLoaded` shows G17.
let menuLoad: number | null = null
// «Muto» and «A schermo intero» (quick menu tools).
const muted = ref(false)
const fullscreen = ref(false)
// G09: px at the bottom the quick menu's bar covers; the game shrinks above it.
const menuInset = ref(0)
// The open game's `speed.ffval` (Tab, the HUD) and `saves.undo`, with its changes «in prova».
const ffSpeed = ref<Speed>(FF_DEFAULT)
const undoAllowed = ref(true)
// `a11y.toggle`: Backspace turns rewind on and the next press off instead of holding.
const rewindToggle = ref(false)
const rewinding = ref(false)
async function loadGameSettings() {
  if (!game.value) return
  try {
    const rows = (
      await Promise.all(
        ['speed', 'saves', 'a11y'].map(p => emu.settings.page(p, { level: 'session' })),
      )
    ).flat()
    const of = (key: string) => rows.find(r => r.key === key)?.value
    ffSpeed.value = speedFromOption(of('speed.ffval')) ?? FF_DEFAULT
    undoAllowed.value = of('saves.undo') !== false
    rewindToggle.value = of('a11y.toggle') === true
  } catch (e) {
    toasts.value?.push(errorText(e), 'error')
  }
}
// E04: the game whose in-game save was set aside, to open again after restoring a backup.
let brokenOpened: { opened: Opened; title: string; colour: string | null } | null = null

async function gameStarted(info: GameInfo, resumeAuto = false) {
  game.value = info
  speed.value = { kind: 'factor', value: 1 }
  menuOpen.value = false
  paused.value = false
  rewinding.value = false
  undoLoadSlot.value = null
  menuLoad = null
  muted.value = false
  menuInset.value = 0
  flowStarted()
  void loadGameSettings()
  if (resumeAuto) {
    await emu.loadState(AUTO_SLOT)
  } else if (info.resumable) {
    // `saves.resume` = ask: hold the game until the answer.
    await emu.setPaused(true)
    flow.resumeAsk = true
  }
}

/** G01 while the core loads, G02 if it cannot start, E04 if its in-game save is unreadable. */
async function start(
  opened: Opened,
  title: string,
  colour: string | null,
  resumeAuto = false,
): Promise<boolean> {
  error.value = null
  flow.launching = { title, colour }
  try {
    const info =
      opened.kind === 'path'
        ? await emu.openGame(opened.path)
        : await emu.library.open(opened.game.id)
    lastOpened = opened
    await gameStarted(info, resumeAuto)
    return true
  } catch (e) {
    if (isErrorCode(e) && e.code === 'saveUnreadable') {
      brokenOpened = { opened, title, colour }
      flow.saveBroken = { title, backupAt: e.params.backupAt }
    } else {
      flow.startError = { title, reason: errorText(e) }
    }
    return false
  } finally {
    flow.launching = null
  }
}

/** E04 «Usa il backup di …»: the newest backup becomes the in-game save, then the game opens. */
async function useLatestBackup() {
  const b = brokenOpened
  flow.saveBroken = null
  brokenOpened = null
  if (!b) return
  try {
    await emu.saves.restoreLatest(b.opened.kind === 'path' ? b.opened.path : b.opened.game.path)
  } catch (e) {
    flow.startError = { title: b.title, reason: errorText(e) }
    return
  }
  await start(b.opened, b.title, b.colour)
}
/** E04 «Gioca senza salvataggio»: the unreadable save is already set aside, so the game opens
 * without one. */
async function playWithoutSave() {
  const b = brokenOpened
  flow.saveBroken = null
  brokenOpened = null
  if (b) await start(b.opened, b.title, b.colour)
}
function closeBroken() {
  flow.saveBroken = null
  brokenOpened = null
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

/**
 * A game from the library (L01): same flow, and the backend counts its play time. With `slot`
 * (V03 «Apri il gioco e carica») that quick save is loaded once the game runs.
 */
async function openLibraryGame(g: LibraryGame, slot?: number) {
  const started = await start({ kind: 'library', game: g }, g.title, g.colour)
  if (!started || slot === undefined) return
  // The quick save asked for wins over the resume question.
  flow.resumeAsk = false
  await emu.setPaused(false)
  await emu.loadState(slot)
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

/** Carica in the quick menu: loads the slot and resumes at once; `stateLoaded` shows G17. */
async function loadFromMenu(slot: number) {
  menuLoad = slot
  try {
    await emu.loadState(slot)
  } catch (e) {
    menuLoad = null
    toasts.value?.push(errorText(e), 'error')
    return
  }
  await setMenu(false)
}

/** G17: «Caricato lo slot N» with «Annulla» for about 5 s; with a controller the undo is in
 * the quick menu instead (board TV-MenuRapido3-Annulla). */
function loadedToast(slot: number) {
  const text = slot === AUTO_SLOT ? t('toast.autoLoaded') : t('toast.stateLoaded', { slot })
  if (!undoAllowed.value) return toasts.value?.push(text)
  if (inputMode.value === 'controller')
    return toasts.value?.push(text, 'info', { note: t('toast.undoFromMenu'), ms: 5000 })
  toasts.value?.push(text, 'info', {
    ms: 5000,
    action: { label: t('toast.undo'), run: () => void undoLoad() },
  })
}
async function undoLoad() {
  try {
    await emu.undoLoad()
  } catch (e) {
    toasts.value?.push(errorText(e), 'error')
  }
}

/** G11 «Riavvia»: the game starts over and resumes. */
async function restartGame() {
  await emu.reset()
  await setMenu(false)
}

async function setMuted(on: boolean) {
  try {
    await emu.setMuted(on)
    muted.value = on
  } catch (e) {
    toasts.value?.push(errorText(e), 'error')
  }
}

async function toggleFullscreen() {
  fullscreen.value = await emu.toggleFullscreen()
}

/** G09's bar: the canvas keeps clear of it by itself, the native video is told the share. */
function setInset(px: number) {
  menuInset.value = px
  void emu.setVideoInset(px > 0 ? Math.min(0.9, px / window.innerHeight) : 0)
}

/** What a controller combo changed (the combo itself already ran in Rust). */
function onHotkey(h: HotkeyEvent) {
  switch (h.kind) {
    case 'fps':
      showFps.value = !showFps.value
      return
    case 'slot':
      comboSlot.value = h.slot
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

/** F2 / F3: the previous / next slot, the same one L3 + ← → moves. */
async function stepSlot(forward: boolean) {
  try {
    onHotkey({ kind: 'slot', slot: await emu.controls.stepSlot(forward) })
  } catch (e) {
    toasts.value?.push(errorText(e), 'error')
  }
}

/** F12, L3 + B (in Rust) and quick menu › Strumenti: G13 «Schermata salvata» on `screenshotSaved`. */
async function screenshot() {
  try {
    await emu.screenshot()
  } catch (e) {
    toasts.value?.push(errorText(e), 'error')
  }
}

/** Backspace: rewind while held, or on / off at each press with `a11y.toggle`. */
function rewindKey(pressed: boolean, repeat: boolean) {
  if (repeat) return
  const on = rewindToggle.value ? (pressed ? !rewinding.value : rewinding.value) : pressed
  if (on === rewinding.value) return
  rewinding.value = on
  void emu.rewind(on).catch(e => toasts.value?.push(errorText(e), 'error'))
}

/** A key of the keyboard table (keyboard.ts) in game, menu closed. Tab and Backspace last while held. */
function runKey(action: KeyAction, pressed: boolean, repeat: boolean) {
  if (action === 'rewind') return rewindKey(pressed, repeat)
  if (action === 'ff_hold') {
    if (repeat || pressed === turboHeld.value) return
    turboHeld.value = pressed
    void emu.setSpeed(pressed ? ffSpeed.value : speed.value)
    return
  }
  if (!pressed || repeat) return
  switch (action) {
    case 'save':
      return void emu.saveState(comboSlot.value)
    case 'load':
      return void emu.loadState(comboSlot.value)
    case 'prev_slot':
    case 'next_slot':
      return void stepSlot(action === 'next_slot')
    case 'fps':
      showFps.value = !showFps.value
      return
    case 'screenshot':
      return void screenshot()
    default:
      // Esc and F11 are handled before; reserved keys never reach here (`keyShortcut`).
      return
  }
}

function onKey(e: KeyboardEvent, pressed: boolean) {
  if (pressed) showFocusRing()
  const shortcut = keyShortcut(e.code)
  // Backspace let go after the menu opened: the rewind it held still ends.
  if (shortcut?.action === 'rewind' && !pressed && !rewindToggle.value && rewinding.value)
    return rewindKey(false, false)
  if (shortcut?.action === 'menu' && pressed) {
    // In the quick menu Esc is «back»: one level up, and from its main screen the game resumes.
    if (menuOpen.value) return dispatchNav('back')
    return void setMenu(true)
  }
  if (shortcut?.action === 'fullscreen') {
    e.preventDefault()
    if (pressed && !e.repeat) void toggleFullscreen()
    return
  }
  // Menu or shell: arrows move the focus, Enter confirms.
  if (!game.value || menuOpen.value) {
    const nav = pressed ? keyToNav(e) : null
    if (nav) {
      e.preventDefault()
      dispatchNav(nav)
    }
    return
  }
  if (shortcut && shortcut.action !== 'menu') {
    e.preventDefault()
    return runKey(shortcut.action, pressed, e.repeat)
  }
  const button = keymap.value[e.code]
  if (button) {
    e.preventDefault()
    if (!e.repeat) void emu.input(button, pressed)
  }
}

const down = (e: KeyboardEvent) => onKey(e, true)
const up = (e: KeyboardEvent) => onKey(e, false)

async function decideFirstRun() {
  try {
    firstRun.value = await emu.firstRun.show()
  } catch (e) {
    firstRun.value = false
    toasts.value?.push(errorText(e), 'error')
  }
}

onMounted(async () => {
  void decideFirstRun()
  ;[videoMode.value, nativeAvailable.value] = await emu.videoMode()
  applyBackground()
  window.addEventListener('keydown', down)
  window.addEventListener('keyup', up)
  watchInput()
  void loadKeymap(false)
  fullscreen.value = await emu.isFullscreen()
  comboSlot.value = await emu.controls.currentSlot()
  unlisten.push(
    await emu.settings.onChanged(e => {
      if (e.key === 'pad.remap') void loadKeymap(game.value !== null)
      if (e.key.startsWith('speed.') || e.key.startsWith('saves.')) void loadGameSettings()
    }),
    await emu.onMenu(() => {
      usedController()
      showFocusRing()
      void setMenu(!menuOpen.value)
    }),
    await emu.onNav(action => {
      usedController()
      dispatchNav(action)
    }),
    await emu.onHotkey(onHotkey),
    await emu.onEffectError(e => toasts.value?.push(errorText(e), 'error')),
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
      // Slot 0 is the auto quick save: it works silently. In the menu a save shows its own
      // line «Salvato nello slot N · Annulla» instead of a toast.
      if (ev.type === 'stateSaved' && ev.slot !== AUTO_SLOT) {
        undoLoadSlot.value = null
        if (!menuOpen.value) toasts.value?.push(t('toast.stateSaved', { slot: ev.slot }))
      }
      if (ev.type === 'stateLoaded') {
        undoLoadSlot.value = ev.slot
        if (menuLoad === ev.slot) loadedToast(ev.slot)
        else if (ev.slot !== AUTO_SLOT)
          toasts.value?.push(t('toast.stateLoaded', { slot: ev.slot }))
        menuLoad = null
      }
      if (ev.type === 'screenshotSaved') toasts.value?.push(t('toast.screenshotSaved'))
      if (ev.type === 'loadUndone') {
        undoLoadSlot.value = null
        toasts.value?.push(t('toast.loadUndone'))
      }
      if (ev.type === 'error') menuLoad = null
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
  <!-- No game, first start on this device: the first run. -->
  <FirstRun
    v-if="!game && firstRun"
    @done="firstRun = false"
    @error="text => toasts?.push(text, 'error')"
  />

  <!-- No game: the shell (rail, sections). -->
  <AppShell
    v-else-if="!game && firstRun === false"
    v-model="section"
    :error="error"
    @open-file="openGame()"
    @open-game="openLibraryGame"
    @error="text => toasts?.push(text, 'error')"
    @notice="text => toasts?.push(text)"
  />

  <!-- In game: no interface, except discreet indicators and the quick menu. -->
  <template v-else-if="game">
    <GameCanvas v-if="videoMode === 'webview'" :inset-bottom="menuInset" />
    <div class="hud">
      <div
        v-if="turboHeld || speed.kind === 'max' || (speed.kind === 'factor' && speed.value !== 1)"
        class="chip num"
      >
        {{ speedLabel(turboHeld ? ffSpeed : speed) }}
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
      :controller="controller"
      :current-slot="comboSlot"
      :undo-load-slot="undoAllowed ? undoLoadSlot : null"
      :muted="muted"
      :fullscreen="fullscreen"
      @video-mode="setVideoMode"
      @resume="setMenu(false)"
      @speed="setSpeed"
      @save="(slot: number) => emu.saveState(slot)"
      @load="loadFromMenu"
      @undo-load="undoLoad"
      @reset="restartGame"
      @close="closeGame"
      @mute="setMuted"
      @fullscreen="toggleFullscreen"
      @screenshot="screenshot"
      @inset="setInset"
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
    @broken-use="useLatestBackup"
    @broken-play="playWithoutSave"
    @broken-close="closeBroken"
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
