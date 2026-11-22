<script setup lang="ts">
// One control of the touch layout, placed where the layout puts it: its centre at the layout's
// fractions of the area, scaled by its size, faded by its own opacity times `touch.topac` (the
// fade on the control as a whole, so a stroke icon stays even). The part itself keeps the design
// system's size (`size-dpad`, the Pad sizes…), so the screens measure its box from the element
// (`data-touch-control`, `offsetWidth` ignores the scale).
// Playing (`interactive`): a button reports its presses, the d-pad its directions, the stick its
// place, the menu opens the quick menu. In the C06 editor it only draws, ringed in the accent
// when it is the chosen one; the editor takes the pointer on the wrapper.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Button as GameButton, TouchControl } from '@/emu'
import { Button, ButtonSize } from '@/components/ui/button'
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
const SIZE: Partial<Record<TouchControl['id'], ButtonSize>> = {
  l: ButtonSize.PadShoulder,
  r: ButtonSize.PadShoulder,
  a: ButtonSize.PadFace,
  b: ButtonSize.PadFace,
  x: ButtonSize.PadFace,
  y: ButtonSize.PadFace,
  start: ButtonSize.PadSystem,
  select: ButtonSize.PadSystem,
}

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
         the game); in the editor the wrapper takes it. -->
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
        <svg
          viewBox="0 0 24 24"
          width="20"
          height="20"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          aria-hidden="true"
        >
          <line x1="4" y1="6" x2="20" y2="6" />
          <line x1="4" y1="12" x2="20" y2="12" />
          <line x1="4" y1="18" x2="20" y2="18" />
        </svg>
      </Button>
      <Button
        v-else-if="button"
        :variant="SKIN_BUTTON[skin]"
        :size="SIZE[control.id]"
        :aria-pressed="latched ? 'true' : undefined"
        :tabindex="interactive ? undefined : -1"
        @pointerdown="(e: PointerEvent) => press(e, button!)"
        @pointerup="release(button!)"
        @pointercancel="release(button!)"
      >
        {{ t(`touch.${control.id}`) }}
      </Button>
    </div>
  </div>
</template>
