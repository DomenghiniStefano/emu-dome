<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { I18nT, useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { emu, KEYMAP, type GameInfo, type Speed, type Stats, type VideoMode } from './emu'
import { errorText, gamepadName, speedLabel } from './i18n/labels'
import GameCanvas from './components/GameCanvas.vue'
import QuickMenu from './components/QuickMenu.vue'
import ToastStack from './components/ToastStack.vue'
import TouchControls from './components/TouchControls.vue'

const { t, n } = useI18n()

const game = ref<GameInfo | null>(null)
const menuOpen = ref(false)
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

const turboHeld = ref(false)
let statsTimer: number | undefined
const unlisten: Array<() => void> = []

async function openGame(given?: string) {
  error.value = null
  const path =
    given ??
    (await open({
      multiple: false,
      filters: [{ name: t('start.dialogFilter'), extensions: ['gba', 'gb', 'gbc'] }],
    }))
  if (typeof path !== 'string') return
  try {
    game.value = await emu.openGame(path)
    speed.value = { kind: 'factor', value: 1 }
    menuOpen.value = false
  } catch (e) {
    error.value = errorText(e)
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
  await emu.setPaused(openMenu)
}

async function setSpeed(s: Speed) {
  speed.value = s
  await emu.setSpeed(s)
}

function onKey(e: KeyboardEvent, pressed: boolean) {
  if (e.code === 'Escape' && pressed) return void setMenu(!menuOpen.value)
  if (!game.value || menuOpen.value) return
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
    await emu.onMenu(() => setMenu(!menuOpen.value)),
    await emu.onGamepadConnected(p =>
      toasts.value?.push(t('toast.gamepadConnected', { name: gamepadName(p) })),
    ),
    await emu.onGamepadDisconnected(p =>
      toasts.value?.push(t('toast.gamepadDisconnected', { name: gamepadName(p) })),
    ),
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
  <!-- No game: the Phase 0 start screen. -->
  <main v-if="!game" class="start">
    <section class="glass-thick start-card">
      <div class="overline">{{ t('start.overline') }}</div>
      <h1>EmuDome</h1>
      <p class="lead">{{ t('start.lead') }}</p>
      <button class="btn btn-primary" @click="openGame()">{{ t('start.openGame') }}</button>
      <p v-if="error" class="error">{{ error }}</p>
      <I18nT keypath="start.hint" tag="p" class="hint">
        <template #path><code>tools/test-rom/emudome-test.gba</code></template>
      </I18nT>
    </section>
  </main>

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
      <div v-if="showFps && stats" class="chip num">
        {{
          t('hud.fps', {
            fps: n(stats.emuFps, { minimumFractionDigits: 1, maximumFractionDigits: 1 }),
          })
        }}
      </div>
    </div>
    <TouchControls v-if="touch && !menuOpen" @menu="setMenu(true)" />
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

  <ToastStack ref="toasts" />
</template>

<style scoped>
.start {
  height: 100%;
  display: grid;
  place-items: center;
  padding: 32px;
}
.start-card {
  width: min(460px, 100%);
  padding: 32px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: flex-start;
}
h1 {
  margin: 0;
  font-size: 32px;
  line-height: 38px;
  font-weight: 600;
  letter-spacing: -0.02em;
}
.lead {
  margin: 0 0 8px;
  font-size: 15px;
  line-height: 22px;
  color: var(--ink-muted);
}
.hint {
  margin: 8px 0 0;
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.error {
  margin: 0;
  color: var(--danger);
}
code {
  font-size: 12px;
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
