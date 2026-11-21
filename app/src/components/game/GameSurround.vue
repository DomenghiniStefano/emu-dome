<script setup lang="ts">
// «Sfondo ai lati del gioco» (`video.surround*`, Settings board › Video) while a game runs: the
// space around the game filled with the cover's colour, a built-in background (optionally
// moving slowly), or the user's image, under «Oscura lo sfondo». Values are the open game's
// (session level, so console and game values apply), followed live.
//
// Native video: a full-window layer with a hole where video.rs draws the game (surround.ts
// mirrors its rectangle); webview mode: under the canvas, whose stage is then transparent.
// «Nero» and «Gioco sfocato» (not available yet, X05) draw nothing: the black of the video stays.
// Decoration only: no input, hidden from assistive technology.
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { coverColours, type CoverColours } from '@/lib/ambient'
import { settingsUnder, useSettingsChanged } from '@/composables/useSettingsChanged'
import { reducedMotion, uiZoom } from '@/lib/appearance'
import { shiftLightness } from '@/lib/colour'
import { insetShare } from './session'
import { backgroundUrl, emu, type LibraryGame, type SettingValue, type VideoMode } from '@/emu'
import {
  AURORA,
  BUILTIN_BACKGROUNDS,
  coverBackground,
  holeClipPath,
  nativeGameRect,
  SURROUND_CARDS,
} from '@/lib/surround'

const props = defineProps<{
  mode: VideoMode
  /** The game's frame size (the core's `started` event); null until known. */
  gameSize: { w: number; h: number } | null
  /** Quick menu G09: screen px at the bottom the menu's bar keeps clear. */
  insetBottom: number
  /** The library game being played, for «Colore della copertina»; null for a file. */
  game: LibraryGame | null
}>()

const values = ref<Record<string, SettingValue | null>>({})
const colours = ref<CoverColours | null>(null)
const win = ref({ w: window.innerWidth, h: window.innerHeight, dpr: window.devicePixelRatio })

async function load() {
  try {
    const rows = await emu.settings.page('video', { level: 'session' })
    values.value = Object.fromEntries(
      rows.filter(r => r.key.startsWith('video.surround')).map(r => [r.key, r.value]),
    )
  } catch (e) {
    console.warn('side background settings unavailable, keeping black', e)
    values.value = {}
  }
  if (props.game && values.value['video.surround'] === 'cover')
    colours.value = await coverColours(props.game)
}

function onResize() {
  win.value = { w: window.innerWidth, h: window.innerHeight, dpr: window.devicePixelRatio }
}

onMounted(() => {
  window.addEventListener('resize', onResize)
  void load()
})
useSettingsChanged(settingsUnder('video.surround'), () => void load())
onUnmounted(() => window.removeEventListener('resize', onResize))

const kind = computed(() => String(values.value['video.surround'] ?? 'black'))
const builtin = computed(() => String(values.value['video.surroundSet'] ?? 'aurora'))
const image = computed(() => {
  const v = values.value['video.surroundImg']
  return typeof v === 'string' && v ? backgroundUrl(v) : null
})

/** The background to paint, or null for the plain black of the video. */
const background = computed<string | null>(() => {
  switch (kind.value) {
    case 'cover': {
      const c = colours.value
      return c ? coverBackground(c[0], shiftLightness(c[0], -0.22)) : SURROUND_CARDS.cover
    }
    case 'builtin':
      return BUILTIN_BACKGROUNDS[builtin.value] ?? AURORA
    case 'image':
      return image.value ? `center / cover no-repeat url("${image.value}"), #000` : null
    default:
      // `black`, and `blur` until the video draws it (shown disabled in the settings).
      return null
  }
})
const dim = computed(() => {
  const v = Number(values.value['video.surroundDim'] ?? 40)
  return Number.isFinite(v) ? Math.min(0.8, Math.max(0, v / 100)) : 0.4
})
const animated = computed(
  () =>
    kind.value === 'builtin' && values.value['video.surroundAnim'] === true && !reducedMotion.value,
)

/** Native video: everything but the game's rectangle (layer px = screen px / zoom). */
const clip = computed(() => {
  if (props.mode !== 'native' || !props.gameSize) return undefined
  const { w, h, dpr } = win.value
  const inset = insetShare(props.insetBottom, h)
  const r = nativeGameRect(
    { w: Math.round(w * dpr), h: Math.round(h * dpr) },
    props.gameSize,
    inset,
  )
  const k = dpr * uiZoom.value
  return holeClipPath({ x: r.x / k, y: r.y / k, w: r.w / k, h: r.h / k }, k)
})
// Under the native video nothing is drawn until the game's rectangle is known.
const shown = computed(
  () => background.value !== null && (props.mode === 'webview' || clip.value !== undefined),
)
</script>

<template>
  <div
    v-if="shown"
    class="pointer-events-none fixed inset-0 overflow-hidden bg-letterbox"
    aria-hidden="true"
    :style="{ clipPath: clip }"
  >
    <div
      class="absolute"
      :class="animated ? '-inset-1/5 animate-surround-drift' : 'inset-0'"
      :style="{ background: background ?? undefined }"
    ></div>
    <div class="absolute inset-0 bg-letterbox" :style="{ opacity: dim }"></div>
  </div>
</template>
