<script setup lang="ts">
// The touch d-pad: one area, the direction follows the finger (diagonals included, dpad.ts). Its
// geometry is the touch board's (`size-dpad`, arms 48 × 52 at `dpad-arm` from the edges). Not
// `interactive` (the C06 editor), it only draws.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Button as GameButton } from '@/emu'
import { dpadDirections } from '@/components/game/dpad'
import { SKIN_PART, type TouchSkin } from './skins'

const props = defineProps<{ skin: TouchSkin; interactive: boolean }>()
const emit = defineEmits<{ dirs: [GameButton[]] }>()
const { t } = useI18n()

const root = ref<HTMLElement | null>(null)
const held = ref<GameButton[]>([])

function set(next: GameButton[]) {
  held.value = next
  emit('dirs', next)
}

function move(e: PointerEvent) {
  const el = root.value
  if (!props.interactive || !el || !el.hasPointerCapture(e.pointerId)) return
  const r = el.getBoundingClientRect()
  set(
    dpadDirections(
      ((e.clientX - r.left) / r.width) * 2 - 1,
      ((e.clientY - r.top) / r.height) * 2 - 1,
    ),
  )
}

function down(e: PointerEvent) {
  if (!props.interactive) return
  root.value?.setPointerCapture(e.pointerId)
  move(e)
}

function up() {
  if (held.value.length > 0) set([])
}

/** The arms, each on its side of the cross. */
const ARMS: { dir: GameButton; place: string }[] = [
  { dir: 'up', place: 'top-0 left-dpad-arm h-52 w-48' },
  { dir: 'down', place: 'bottom-0 left-dpad-arm h-52 w-48' },
  { dir: 'left', place: 'top-dpad-arm left-0 h-48 w-52' },
  { dir: 'right', place: 'top-dpad-arm right-0 h-48 w-52' },
]
</script>

<template>
  <div
    ref="root"
    class="relative size-dpad touch-none"
    role="group"
    :aria-label="t('touch.dpad')"
    @pointerdown="down"
    @pointermove="move"
    @pointerup="up"
    @pointercancel="up"
  >
    <span
      v-for="arm in ARMS"
      :key="arm.dir"
      class="absolute rounded-6"
      :class="[arm.place, held.includes(arm.dir) ? SKIN_PART[skin].held : SKIN_PART[skin].rest]"
    />
  </div>
</template>
