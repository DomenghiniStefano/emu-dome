<script setup lang="ts">
// "webview" mode: the game drawn in a canvas inside the interface.
// Used where native video cannot sit under the webview (Linux) and to compare the two paths.
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { uiZoom } from '../appearance'
import { feColorMatrix, isColourBlindMode } from '../colourBlind'
import { emu, frameUrl, type BenchFrame } from '../emu'

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
const style = ref<Record<string, string>>({})
let seq = -1
let running = true
let size = { w: 0, h: 0 }

// Bench mode: every round of the loop is recorded and sent to Rust once a second,
// so the bench report measures the webview path like the native one.
let benchFrames: BenchFrame[] | null = null
let benchTimer: number | undefined
function benchRecord(drawn: boolean) {
  benchFrames?.push({ t: performance.timeOrigin + performance.now(), seq, drawn })
}
function benchFlush() {
  if (!benchFrames?.length) return
  void emu.benchFrames(benchFrames)
  benchFrames = []
}

// Sizes come from the window, in screen px; the interface may be scaled (`a11y.textsize`, CSS
// zoom on <html>), the game is not: px written in styles are divided by the zoom.
function layout() {
  if (!size.w) return
  const height = Math.max(1, window.innerHeight - (props.insetBottom ?? 0))
  const fit = Math.min(window.innerWidth / size.w, height / size.h)
  const scale = (fit >= 1 ? Math.floor(fit) : fit) / uiZoom.value
  style.value = { width: `${size.w * scale}px`, height: `${size.h * scale}px` }
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
      const c = canvas.value
      if (c.width !== w || c.height !== h) {
        c.width = w
        c.height = h
        size = { w, h }
        layout()
      }
      c.getContext('2d')!.putImageData(new ImageData(data, w, h), 0, 0)
      benchRecord(true)
    } else {
      benchRecord(false)
    }
  } catch {
    // frame lost: try again on the next round
  }
  requestAnimationFrame(tick)
}

watch(() => props.insetBottom, layout)
watch(uiZoom, layout)

onMounted(async () => {
  window.addEventListener('resize', layout)
  requestAnimationFrame(tick)
  if (await emu.benchActive()) {
    benchFrames = []
    benchTimer = window.setInterval(benchFlush, 1000)
  }
})
onUnmounted(() => {
  running = false
  window.removeEventListener('resize', layout)
  window.clearInterval(benchTimer)
})
</script>

<template>
  <div
    class="fixed inset-0 grid place-items-center"
    :class="surround ? 'bg-transparent' : 'bg-letterbox'"
    :style="{ bottom: `${(insetBottom ?? 0) / uiZoom}px` }"
  >
    <svg v-if="filter" class="absolute size-0" aria-hidden="true">
      <filter :id="filter.id" color-interpolation-filters="sRGB">
        <feColorMatrix type="matrix" :values="filter.values" />
      </filter>
    </svg>
    <canvas
      ref="canvas"
      class="[image-rendering:pixelated]"
      :style="{ ...style, filter: filter ? `url(#${filter.id})` : undefined }"
      :aria-label="t('canvas.label')"
    />
  </div>
</template>
