<script setup lang="ts">
// The 3DS Circle Pad drawn on screen (Trello #29): a ring with a knob that follows the finger;
// the knob's offset from the centre, -1..1 each way (y downwards), is the left stick. Let go, it
// springs back to the centre. Drawn in the style's colours (skins.ts), the knob lit while moved.
// Not `interactive` (the C06 editor), it only draws.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { SKIN_PART, type TouchSkin } from './skins'
import { stickAt } from './stick'

const props = defineProps<{ skin: TouchSkin; interactive: boolean }>()
const emit = defineEmits<{ move: [number, number] }>()
const { t } = useI18n()

const root = ref<HTMLElement | null>(null)
const at = ref<[number, number]>([0, 0])
const held = computed(() => at.value[0] !== 0 || at.value[1] !== 0)

function move(e: PointerEvent) {
  const el = root.value
  if (!props.interactive || !el || !el.hasPointerCapture(e.pointerId)) return
  const r = el.getBoundingClientRect()
  at.value = stickAt(
    ((e.clientX - r.left) / r.width) * 2 - 1,
    ((e.clientY - r.top) / r.height) * 2 - 1,
  )
  emit('move', at.value[0], at.value[1])
}

function down(e: PointerEvent) {
  if (!props.interactive) return
  root.value?.setPointerCapture(e.pointerId)
  move(e)
}

function up() {
  at.value = [0, 0]
  emit('move', 0, 0)
}

/** In the box's CSS pixels (spacing.css `pad-stick`, 104): the ring, the knob (44 across) and how
 * far its centre travels, the ring's radius less its own. */
const BOX = 104
const EDGE = 1.5
const KNOB = 22
const TRAVEL = BOX / 2 - KNOB
const knob = computed(() => ({
  cx: BOX / 2 + at.value[0] * TRAVEL,
  cy: BOX / 2 + at.value[1] * TRAVEL,
}))
</script>

<template>
  <div
    ref="root"
    class="relative size-pad-stick touch-none rounded-full"
    role="group"
    :aria-label="t('touch.circlePad')"
    @pointerdown="down"
    @pointermove="move"
    @pointerup="up"
    @pointercancel="up"
  >
    <svg
      class="pointer-events-none absolute inset-0 size-full overflow-visible"
      :class="SKIN_PART[skin].base"
      :viewBox="`0 0 ${BOX} ${BOX}`"
      aria-hidden="true"
    >
      <circle :cx="BOX / 2" :cy="BOX / 2" :r="BOX / 2 - EDGE / 2" :stroke-width="EDGE" />
      <circle
        v-bind="knob"
        :r="KNOB - EDGE / 2"
        :stroke-width="EDGE"
        :class="held ? SKIN_PART[skin].held : ''"
      />
    </svg>
  </div>
</template>
