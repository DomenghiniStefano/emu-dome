<script setup lang="ts">
// One screen in the C09 editor: a frame where the game draws that screen (or, with no game open,
// a placeholder with its name). Drag it to move it; the selected one is ringed in the accent and
// has the accent corner that resizes it. Focused (L1 / R1, a click, Tab), the d-pad and the arrows
// move it (the editor's own steps, useScreenEditor).
import { computed } from 'vue'
import type { CustomScreen } from '@/emu'
import { SCREEN_ATTR } from './screenEditor'

const props = defineProps<{
  screen: CustomScreen
  /** The name it shows («Superiore», «Touch»): the picture it holds, swapped or not. */
  label: string
  selected: boolean
  /** No game open: drawn as a placeholder with its name. */
  placeholder: boolean
}>()
const emit = defineEmits<{
  /** A press on the frame (`corner`: on its resize corner). */
  press: [PointerEvent, boolean]
  focus: []
}>()

const pct = (v: number) => `${v * 100}%`
const box = computed(() => ({
  left: pct(props.screen.x),
  top: pct(props.screen.y),
  width: pct(props.screen.w),
  height: pct(props.screen.h),
}))
const mark = computed(() => ({ [SCREEN_ATTR]: props.screen.id }))
/** The resize corner kept inside the window (#140): a frame past the window's right or bottom
 * edge (the editors' frames are fractions of the window) moves it in by what is past. */
const corner = computed(() => {
  const { x, y, w, h } = props.screen
  const past = (end: number, size: number) => (size > 0 ? pct(Math.max(0, end - 1) / size) : '0%')
  return { right: past(x + w, w), bottom: past(y + h, h) }
})
</script>

<template>
  <div
    v-bind="mark"
    tabindex="0"
    :aria-label="label"
    :aria-current="selected ? 'true' : undefined"
    class="absolute cursor-grab touch-none focus-ring-none select-none"
    :class="
      selected
        ? 'outline-solid outline-ring outline-offset-ring outline-accent'
        : 'outline outline-line-strong'
    "
    :style="box"
    @pointerdown="e => emit('press', e, false)"
    @focus="emit('focus')"
  >
    <span
      v-if="placeholder"
      class="flex size-full items-center justify-center bg-surface-raised font-semibold text-ink-muted"
    >
      {{ label }}
    </span>
    <!-- At the bottom: the editors' captions and panels stand at the top of the window (#140). -->
    <span
      v-else
      class="absolute bottom-6 left-8 rounded-badge border border-line bg-glass-thick px-6 text-caption font-semibold"
    >
      {{ label }}
    </span>
    <span
      v-if="selected"
      class="absolute size-24 cursor-nwse-resize rounded-tl-control bg-accent"
      :style="corner"
      aria-hidden="true"
      @pointerdown.stop="e => emit('press', e, true)"
    ></span>
  </div>
</template>
