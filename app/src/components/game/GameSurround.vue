<script setup lang="ts">
// «Sfondo ai lati del gioco» (`video.surround*`, Settings board › Video) while a game runs: the
// space around the game filled with the cover's colour, a built-in background (optionally
// moving slowly), or the user's image, under «Oscura lo sfondo». Values are the open game's
// (session level, so console and game values apply), followed live.
//
// Native video: a full-window layer with one hole per screen where video.rs draws it, as Rust's
// `screens_placements` says (two screens apart on a phone leave the room between them painted;
// the interface works out no geometry); webview mode: under the canvas, whose stage is then
// transparent.
// «Nero» and «Gioco sfocato» (not available yet, X05) draw nothing: the black of the video stays.
// Decoration only: no input, hidden from assistive technology.
import { computed, onMounted, onUnmounted, ref, shallowRef, watch } from 'vue'
import { coverColours, type CoverColours } from '@/lib/ambient'
import { settingsUnder, useSettingsChanged } from '@/composables/useSettingsChanged'
import { reducedMotion, uiZoom } from '@/lib/appearance'
import { shiftLightness } from '@/design-system/lib/colour'
import { latest } from '@/lib/latest'
import { insetShares, type InsetPx } from './inset'
import {
  backgroundUrl,
  emu,
  type LibraryGame,
  type ScreenRect,
  type SettingValue,
  type VideoMode,
} from '@/emu'
import {
  AURORA,
  BUILTIN_BACKGROUNDS,
  coverBackground,
  holesClipPath,
  SURROUND_CARDS,
} from '@/lib/surround'

const props = defineProps<{
  mode: VideoMode
  /** The game's frame size (the core's `started` event); null until known. */
  gameSize: { w: number; h: number } | null
  /** The edges the quick menu or the touch controls keep clear (screen px). */
  inset: InsetPx
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

// Native video: where video.rs draws each screen, in physical pixels, as Rust composes them
// (`screens_placements` for the window's surface and the inset: the same `screens::compose` the
// native renderer runs). Asked again when the window, the inset, the frame or the layout changes
// (the swap, the layout shown, a `screens.*` setting: `emu.screens.onChanged`).
const screens = shallowRef<ScreenRect[] | null>(null)
const placements = latest()
/** Before the core's first frame Rust has nothing to place: asked again shortly. */
const RETRY_MS = 200
let retry: number | undefined
let alive = true

async function compose() {
  window.clearTimeout(retry)
  if (props.mode !== 'native' || !props.gameSize) {
    placements.next()
    screens.value = null
    return
  }
  const fresh = placements.next()
  const { w, h, dpr } = win.value
  // The surface is whole device pixels.
  const area = {
    width: Math.round(w * dpr),
    height: Math.round(h * dpr),
    inset: insetShares(props.inset, w, h),
  }
  try {
    const res = await emu.screens.placements(area)
    if (!fresh() || !alive) return
    screens.value = res.placements.length ? res.placements.map(p => p.dst) : null
    if (!res.placements.length) retry = window.setTimeout(() => void compose(), RETRY_MS)
  } catch (e) {
    // Decoration only: without the screens' places the layer stays off and the video's black
    // shows, never a background over the game.
    if (fresh()) {
      console.warn('side background: screen placements unavailable, keeping black', e)
      screens.value = null
    }
  }
}
watch(
  () => [props.mode, props.gameSize, props.inset, win.value] as const,
  () => void compose(),
  { immediate: true },
)
let stopScreens: (() => void) | null = null
onMounted(async () => {
  stopScreens = await emu.screens.onChanged(() => void compose())
  if (!alive) stopScreens()
})
onUnmounted(() => {
  alive = false
  window.clearTimeout(retry)
  stopScreens?.()
})

/** Native video: everything but the screens (layer px = screen px / zoom). */
const clip = computed(() => {
  if (props.mode !== 'native' || !screens.value) return undefined
  const k = win.value.dpr * uiZoom.value
  return holesClipPath(
    screens.value.map(r => ({ x: r.x / k, y: r.y / k, w: r.w / k, h: r.h / k })),
    k,
  )
})
// Under the native video nothing is drawn until the screens' places are known.
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
