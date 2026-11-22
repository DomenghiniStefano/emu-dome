<script setup lang="ts">
// The 3DS Circle Pad drawn on screen (Trello #29): a ring with a knob that follows the finger;
// the knob's offset from the centre, -1..1 each way (y downwards), is the left stick. Let go, it
// springs back to the centre. Not `interactive` (the C06 editor), it only draws.
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

/** The knob's place: its centre moves over the ring's radius less its own. */
const knob = computed(() => ({
  transform: `translate(calc(-50% + ${at.value[0]} * (var(--spacing-pad-stick) - var(--spacing-pad-knob)) / 2), calc(-50% + ${at.value[1]} * (var(--spacing-pad-stick) - var(--spacing-pad-knob)) / 2))`,
}))
</script>

<template>
  <div
    ref="root"
    class="relative size-pad-stick touch-none rounded-full"
    :class="SKIN_PART[skin].rest"
    role="group"
    :aria-label="t('touch.circlePad')"
    @pointerdown="down"
    @pointermove="move"
    @pointerup="up"
    @pointercancel="up"
  >
    <span
      class="absolute top-1/2 left-1/2 size-pad-knob rounded-full"
      :class="held ? SKIN_PART[skin].held : SKIN_PART[skin].rest"
      :style="knob"
    />
  </div>
</template>
