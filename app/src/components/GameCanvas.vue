<script setup lang="ts">
// "webview" mode: the game drawn in a canvas inside the interface.
// Used where native video cannot sit under the webview (Linux) and to compare the two paths.
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { frameUrl } from '../emu'

const props = defineProps<{
  /** Quick menu G09: px at the bottom kept clear for the menu's bar; the game shrinks above. */
  insetBottom?: number
}>()

const { t } = useI18n()

const canvas = ref<HTMLCanvasElement | null>(null)
const style = ref<Record<string, string>>({})
let seq = -1
let running = true
let size = { w: 0, h: 0 }

function layout() {
  if (!size.w) return
  const height = Math.max(1, window.innerHeight - (props.insetBottom ?? 0))
  const fit = Math.min(window.innerWidth / size.w, height / size.h)
  const scale = fit >= 1 ? Math.floor(fit) : fit
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
    }
  } catch {
    // frame lost: try again on the next round
  }
  requestAnimationFrame(tick)
}

watch(() => props.insetBottom, layout)

onMounted(() => {
  window.addEventListener('resize', layout)
  requestAnimationFrame(tick)
})
onUnmounted(() => {
  running = false
  window.removeEventListener('resize', layout)
})
</script>

<template>
  <div class="stage" :style="{ bottom: `${insetBottom ?? 0}px` }">
    <canvas ref="canvas" :style="style" :aria-label="t('canvas.label')" />
  </div>
</template>

<style scoped>
.stage {
  position: fixed;
  inset: 0;
  background: #000;
  display: grid;
  place-items: center;
}
canvas {
  image-rendering: pixelated;
}
</style>
