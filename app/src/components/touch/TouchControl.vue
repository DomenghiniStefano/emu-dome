<script setup lang="ts">
// One control of the touch layout, placed where the layout puts it: its centre at the layout's
// fractions of the area, scaled by its size, faded as a whole by the style (`opacity-pad`) times
// `touch.topac` times its own opacity (one fade per control over opaque colours, so an edge never
// darkens at a joint). The part itself keeps the design system's size (`size-dpad`, the Pad
// sizes…), so the screens measure its box from the element (`data-touch-control`, `offsetWidth`
// ignores the scale); the box is what takes the touch, the shape drawn in it may be smaller
// (PadShape.vue).
// Playing (`interactive`): a button reports its presses, the d-pad its directions, the stick its
// place, the menu opens the quick menu. In the C06 editor it only draws, ringed in the accent
// when it is the chosen one; the editor takes the pointer on the wrapper.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Button as GameButton, TouchControl } from '@/emu'
import { Button, ButtonSize } from '@/design-system/components/button'
import PadShape, { type PadShapeKind } from './PadShape.vue'
import TouchDpad from './TouchDpad.vue'
import TouchStick from './TouchStick.vue'
import { SKIN_BUTTON, type TouchSkin } from './skins'
import { gameButton } from './touchControls'

const props = defineProps<{
  control: TouchControl
  skin: TouchSkin
  /** `touch.topac`, per cent. */
  opacity: number
  interactive: boolean
  /** A «hold» button held down. */
  latched?: boolean
  /** The C06 editor's chosen control. */
  selected?: boolean
}>()
const emit = defineEmits<{
  down: [GameButton]
  up: [GameButton]
  dirs: [GameButton[]]
  stick: [number, number]
  menu: []
}>()
const { t } = useI18n()

const button = computed(() => gameButton(props.control.id))
/** Each button's box and the shape drawn in it. */
const LOOK: Partial<Record<TouchControl['id'], { size: ButtonSize; shape: PadShapeKind }>> = {
  l: { size: ButtonSize.PadShoulder, shape: 'shoulderL' },
  r: { size: ButtonSize.PadShoulder, shape: 'shoulderR' },
  a: { size: ButtonSize.PadFace, shape: 'face' },
  b: { size: ButtonSize.PadFace, shape: 'face' },
  x: { size: ButtonSize.PadFace, shape: 'face' },
  y: { size: ButtonSize.PadFace, shape: 'face' },
  start: { size: ButtonSize.PadSystem, shape: 'pill' },
  select: { size: ButtonSize.PadSystem, shape: 'pill' },
}
const look = computed(() => LOOK[props.control.id])

const place = computed(() => ({
  left: `${props.control.x * 100}%`,
  top: `${props.control.y * 100}%`,
  transform: `translate(-50%, -50%) scale(${props.control.size / 100})`,
}))
const fade = computed(() => ({ opacity: (props.opacity * props.control.opacity) / 10_000 }))
const mark = computed(() => ({ 'data-touch-control': props.control.id }))

function press(e: PointerEvent, b: GameButton) {
  if (!props.interactive) return
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  emit('down', b)
}
function release(b: GameButton) {
  if (props.interactive) emit('up', b)
}
</script>

<template>
  <div
    v-bind="mark"
    class="absolute"
    :class="[
      interactive ? 'pointer-events-none' : 'pointer-events-auto cursor-grab touch-none',
      selected ? 'rounded-card outline-solid outline-ring outline-offset-ring outline-accent' : '',
    ]"
    :style="place"
  >
    <!-- Playing, only the part itself takes a touch (a round button's corners let it through to
         the game); in the editor the wrapper takes it. The style's fade, then the user's. -->
    <div class="opacity-pad">
      <div :style="fade" :class="interactive ? '*:pointer-events-auto' : '*:pointer-events-none'">
        <TouchDpad
          v-if="control.id === 'dpad'"
          :skin="skin"
          :interactive="interactive"
          @dirs="d => emit('dirs', d)"
        />
        <TouchStick
          v-else-if="control.id === 'circlePad'"
          :skin="skin"
          :interactive="interactive"
          @move="(x, y) => emit('stick', x, y)"
        />
        <Button
          v-else-if="control.id === 'menu'"
          :variant="SKIN_BUTTON[skin]"
          :size="ButtonSize.PadMenu"
          :aria-label="t('touch.menu')"
          :tabindex="interactive ? undefined : -1"
          @click="interactive && emit('menu')"
        >
          <PadShape kind="menu" />
        </Button>
        <Button
          v-else-if="button && look"
          :variant="SKIN_BUTTON[skin]"
          :size="look.size"
          :aria-pressed="latched ? 'true' : undefined"
          :tabindex="interactive ? undefined : -1"
          @pointerdown="(e: PointerEvent) => press(e, button!)"
          @pointerup="release(button!)"
          @pointercancel="release(button!)"
        >
          <PadShape :kind="look.shape" />
          <span class="relative">{{ t(`touch.${control.id}`) }}</span>
        </Button>
      </div>
    </div>
  </div>
</template>
