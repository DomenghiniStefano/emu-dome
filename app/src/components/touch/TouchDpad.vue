<script setup lang="ts">
// The touch d-pad (owner, 2026-11-22): one cross-shaped piece with an arrow in each arm
// (padGeometry.ts), in the style's colours (skins.ts); a held direction lights its arm. The whole
// box takes the touch and the direction follows the finger (dpad.ts): between two arms, both
// directions. Not `interactive` (the C06 editor), it only draws.
import { ref, useId } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Button as GameButton } from '@/emu'
import { dpadDirections } from '@/components/game/dpad'
import { DPAD_ARMS, DPAD_BOX, DPAD_CROSS, DPAD_EDGE } from './padGeometry'
import { SKIN_PART, type TouchSkin } from './skins'

const props = defineProps<{ skin: TouchSkin; interactive: boolean }>()
const emit = defineEmits<{ dirs: [GameButton[]] }>()
const { t } = useI18n()

const root = ref<HTMLElement | null>(null)
const held = ref<GameButton[]>([])
/** The cross as a clip for the lit arms (one per d-pad on the page). */
const clip = `dpad-${useId()}`

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
    <svg
      class="pointer-events-none absolute inset-0 size-full overflow-visible"
      :class="SKIN_PART[skin].base"
      :viewBox="`0 0 ${DPAD_BOX} ${DPAD_BOX}`"
      aria-hidden="true"
    >
      <defs>
        <clipPath :id="clip"><path :d="DPAD_CROSS" /></clipPath>
      </defs>
      <path :d="DPAD_CROSS" stroke="none" />
      <g :clip-path="`url(#${clip})`" stroke="none">
        <template v-for="arm in DPAD_ARMS" :key="arm.dir">
          <rect v-if="held.includes(arm.dir)" v-bind="arm.rect" :class="SKIN_PART[skin].held" />
        </template>
      </g>
      <path :d="DPAD_CROSS" fill="none" :stroke-width="DPAD_EDGE" stroke-linejoin="round" />
      <g fill="currentColor" stroke="none">
        <polygon v-for="arm in DPAD_ARMS" :key="arm.dir" :points="arm.arrow" />
      </g>
    </svg>
  </div>
</template>
