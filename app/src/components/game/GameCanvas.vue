<script setup lang="ts">
// "webview" mode: the game drawn in a canvas inside the interface.
// Used where native video cannot sit under the webview (Linux) and to compare the two paths.
// Rust composes the screens (`screens.*`, swapped, turned; one screen for GB / GBC / GBA) and
// says where each goes (`emu.screens.placements`); this canvas only draws them there
// (screenDraw.ts), over the whole window, in device pixels.
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { uiZoom } from '@/lib/appearance'
import { feColorMatrix, isColourBlindMode } from './colourBlind'
import { screenDraws, type ScreenDraw } from './screenDraw'
import { viewArea } from './screenTouch'
import { emu, frameUrl, type BenchFrame } from '@/emu'
import { ignore, reportError } from '@/lib/errors'

const props = defineProps<{
  /** Quick menu G09: px at the bottom kept clear for the menu's bar; the game shrinks above. */
  insetBottom?: number
  /** `a11y.cblind`: the colour correction drawn on the game (`none` or absent: none). */
  colourBlind?: string
  /** A side background (GameSurround) is painted under the stage: no black of its own. */
  surround?: boolean
}>()

// The filter's id and matrix; none for `none`.
const filter = computed(() =>
  isColourBlindMode(props.colourBlind)
    ? { id: `emudome-cblind-${props.colourBlind}`, values: feColorMatrix(props.colourBlind) }
    : null,
)

const { t } = useI18n()

const canvas = ref<HTMLCanvasElement | null>(null)
// The core's frame as it came; each screen is drawn from its part of it.
const frame = document.createElement('canvas')
let seq = -1
let running = true
// Where the screens go, and the frame size Rust composed them for.
let draws: ScreenDraw[] = []
let composedFor = { w: 0, h: 0 }
// Only the newest answer is used (an older one is for another window, layout or frame).
let request = 0

// Bench mode: every round of the loop is recorded and sent to Rust once a second,
// so the bench report measures the webview path like the native one.
let benchFrames: BenchFrame[] | null = null
let benchTimer: number | undefined
function benchRecord(drawn: boolean) {
  benchFrames?.push({ t: performance.timeOrigin + performance.now(), seq, drawn })
}
function benchFlush() {
  if (!benchFrames?.length) return
  // Bench mode only, a measuring tool: a lost batch shows in its report as missing frames.
  ignore(emu.benchFrames(benchFrames), 'bench frames not sent')
  benchFrames = []
}

/** Asks Rust where the screens go now (window, menu bar, layout, frame size). */
async function compose() {
  const mine = ++request
  const area = viewArea(window, 1, props.insetBottom ?? 0)
  const scale = window.devicePixelRatio || 1
  try {
    const res = await emu.screens.placements(area)
    if (mine !== request) return
    draws = screenDraws(res.placements, scale)
    composedFor = { w: res.frameWidth, h: res.frameHeight }
    paint()
  } catch (e) {
    if (mine === request) reportError(e)
  }
}

/** The canvas covers the window in device pixels; the screens are placed again. */
function resize() {
  const c = canvas.value
  if (c) {
    const scale = window.devicePixelRatio || 1
    const w = Math.round(window.innerWidth * scale)
    const h = Math.round(window.innerHeight * scale)
    if (c.width !== w || c.height !== h) {
      c.width = w
      c.height = h
    }
  }
  void compose()
}

/** Each screen from the frame into its rectangle (nothing until they match the frame). */
function paint() {
  const c = canvas.value
  const ctx = c?.getContext('2d')
  if (!c || !ctx) return
  ctx.setTransform(1, 0, 0, 1, 0, 0)
  ctx.clearRect(0, 0, c.width, c.height)
  if (frame.width !== composedFor.w || frame.height !== composedFor.h) return
  ctx.imageSmoothingEnabled = false
  for (const d of draws) {
    ctx.setTransform(1, 0, 0, 1, d.cx, d.cy)
    ctx.rotate(d.angle)
    ctx.drawImage(frame, d.sx, d.sy, d.sw, d.sh, -d.w / 2, -d.h / 2, d.w, d.h)
  }
  ctx.setTransform(1, 0, 0, 1, 0, 0)
}

async function tick() {
  if (!running) return
  try {
    const res = await fetch(`${frameUrl}?since=${seq}`, { cache: 'no-store' })
    if (res.status === 200 && canvas.value) {
      const w = Number(res.headers.get('x-width'))
      const h = Number(res.headers.get('x-height'))
      seq = Number(res.headers.get('x-seq'))
      const data = new Uint8ClampedArray(await res.arrayBuffer())
      if (frame.width !== w || frame.height !== h) {
        frame.width = w
        frame.height = h
        // Another frame size (a new game, the core changed it): its screens again.
        void compose()
      }
      frame.getContext('2d')!.putImageData(new ImageData(data, w, h), 0, 0)
      paint()
      benchRecord(true)
    } else {
      benchRecord(false)
    }
  } catch {
    // frame lost: try again on the next round
  }
  requestAnimationFrame(tick)
}

watch(() => props.insetBottom, resize)
watch(uiZoom, resize)

let stopScreens: (() => void) | null = null
onMounted(async () => {
  window.addEventListener('resize', resize)
  resize()
  // The layout, the swap or a `screens.*` setting changed: the screens move.
  stopScreens = await emu.screens.onChanged(() => void compose())
  if (!running) stopScreens()
  requestAnimationFrame(tick)
  if (await emu.benchActive()) {
    benchFrames = []
    benchTimer = window.setInterval(benchFlush, 1000)
  }
})
onUnmounted(() => {
  running = false
  stopScreens?.()
  window.removeEventListener('resize', resize)
  window.clearInterval(benchTimer)
})
</script>

<template>
  <div
    class="fixed inset-0"
    :class="surround ? 'bg-transparent' : 'bg-letterbox'"
    :style="{ bottom: `${(insetBottom ?? 0) / uiZoom}px` }"
  >
    <svg v-if="filter" class="absolute size-0" aria-hidden="true">
      <filter :id="filter.id" color-interpolation-filters="sRGB">
        <feColorMatrix type="matrix" :values="filter.values" />
      </filter>
    </svg>
  </div>
  <canvas
    ref="canvas"
    class="fixed inset-0 size-full"
    :style="{ filter: filter ? `url(#${filter.id})` : undefined }"
    :aria-label="t('canvas.label')"
  />
</template>
