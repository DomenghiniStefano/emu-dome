<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { Library } from '@lucide/vue'
import {
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
import { errorText, gamepadName, speedLabel } from './i18n/labels'
import { dispatchNav, keyToNav, showFocusRing } from './nav'
import type { Section } from './sections'
import AppShell from './components/AppShell.vue'
import GameCanvas from './components/GameCanvas.vue'
import QuickMenu from './components/QuickMenu.vue'
import ToastStack from './components/ToastStack.vue'
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

function gameStarted(info: GameInfo) {
  game.value = info
  speed.value = { kind: 'factor', value: 1 }
  menuOpen.value = false
  paused.value = false
}

async function openGame(given?: string) {
  error.value = null
  const path =
    given ??
    (await open({
      multiple: false,
      filters: [{ name: t('library.dialogFilter'), extensions: ['gba', 'gb', 'gbc'] }],
    }))
  if (typeof path !== 'string') return
  try {
    gameStarted(await emu.openGame(path))
  } catch (e) {
    error.value = errorText(e)
  }
}

/** A game from the library (L01): same flow, and the backend counts its play time. */
async function openLibraryGame(g: LibraryGame) {
  try {
    gameStarted(await emu.library.open(g.id))
  } catch (e) {
    toasts.value?.push(errorText(e), 'error')
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

async function closeGame() {
  await emu.closeGame()
  game.value = null
  menuOpen.value = false
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
    await emu.onGamepadConnected(p =>
      toasts.value?.push(t('toast.gamepadConnected', { name: gamepadName(p) })),
    ),
    await emu.onGamepadDisconnected(p =>
      toasts.value?.push(t('toast.gamepadDisconnected', { name: gamepadName(p) })),
    ),
    await emu.library.onDragDrop(e => {
      if (e.type === 'over') dragOver.value = !game.value
      else if (e.type === 'leave') dragOver.value = false
      else void onDrop(e.paths)
    }),
    ...(await startLibrary()),
    await emu.onEvent(ev => {
      if (ev.type === 'stateSaved') toasts.value?.push(t('toast.stateSaved', { slot: ev.slot }))
      if (ev.type === 'stateLoaded') toasts.value?.push(t('toast.stateLoaded', { slot: ev.slot }))
      if (ev.type === 'error') toasts.value?.push(errorText(ev.error), 'error')
    }),
  )
  const startup = await emu.startupGame()
  if (startup) await openGame(startup)
  statsTimer = window.setInterval(async () => {
    if (game.value) stats.value = await emu.stats()
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
      @video-mode="setVideoMode"
      @resume="setMenu(false)"
      @speed="setSpeed"
      @save="(slot: number) => emu.saveState(slot)"
      @load="(slot: number) => emu.loadState(slot)"
      @reset="emu.reset()"
      @close="closeGame"
    />
  </template>

  <!-- Drag and drop over the window: games and folders go into the library. -->
  <div v-if="dragOver && !game" class="drop" aria-hidden="true">
    <div class="drop-box">
      <Library :size="44" :stroke-width="1.5" />
      <div class="drop-text">{{ t('library.drop') }}</div>
    </div>
  </div>

  <ToastStack ref="toasts" />
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
