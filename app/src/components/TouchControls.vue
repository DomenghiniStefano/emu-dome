<script setup lang="ts">
// Minimal touch overlay (Phase 0, Android): D-pad, A/B, L/R, Start/Select and the quick menu.
// Each control keeps its own pointer, so several can be held at once. The overlay lets touches
// through except on the controls. Its geometry (d-pad 148, arms 48 x 52, faces 64, shoulders
// 88) is the touch board's own, outside the design system's steps: Cowork places it.
import { onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { emu, type Button as GameButton } from '../emu'
import { dpadDirections } from '../lib/touch/dpad'

/** GB and GBC have no shoulder buttons: L/R are not drawn for them. */
const props = defineProps<{ shoulders: boolean }>()
const emit = defineEmits<{ menu: [] }>()
const { t } = useI18n()

/** Buttons currently held by the overlay, so nothing stays pressed when it goes away. */
const held = new Set<GameButton>()

function press(button: GameButton, pressed: boolean) {
  if (pressed === held.has(button)) return
  if (pressed) held.add(button)
  else held.delete(button)
  void emu.input(button, pressed)
}

function hold(e: PointerEvent, button: GameButton) {
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  press(button, true)
}

// D-pad: one area, the direction follows the finger (diagonals included).
const dpad = ref<HTMLElement | null>(null)
const dirs = ref<GameButton[]>([])

function dpadMove(e: PointerEvent) {
  const el = dpad.value
  if (!el || !el.hasPointerCapture(e.pointerId)) return
  const r = el.getBoundingClientRect()
  setDirs(
    dpadDirections(
      ((e.clientX - r.left) / r.width) * 2 - 1,
      ((e.clientY - r.top) / r.height) * 2 - 1,
    ),
  )
}

function setDirs(next: GameButton[]) {
  for (const b of dirs.value) if (!next.includes(b)) press(b, false)
  for (const b of next) press(b, true)
  dirs.value = next
}

function dpadDown(e: PointerEvent) {
  dpad.value?.setPointerCapture(e.pointerId)
  dpadMove(e)
}

onUnmounted(() => {
  for (const b of [...held]) press(b, false)
})

/** The d-pad's arms, each placed on its side of the cross. */
const ARMS: { dir: GameButton; place: string }[] = [
  { dir: 'up', place: 'top-0 left-[50px] h-[52px] w-48' },
  { dir: 'down', place: 'bottom-0 left-[50px] h-[52px] w-48' },
  { dir: 'left', place: 'top-[50px] left-0 h-48 w-[52px]' },
  { dir: 'right', place: 'top-[50px] right-0 h-48 w-[52px]' },
]
const FACES: { button: GameButton; key: string; place: string }[] = [
  { button: 'b', key: 'touch.b', place: 'bottom-16 left-0' },
  { button: 'a', key: 'touch.a', place: 'top-16 right-0' },
]
const SHOULDERS: { button: GameButton; key: string; place: string }[] = [
  { button: 'l', key: 'touch.l', place: 'left-12' },
  { button: 'r', key: 'touch.r', place: 'right-12' },
]
const SYSTEM: { button: GameButton; key: string }[] = [
  { button: 'select', key: 'touch.select' },
  { button: 'start', key: 'touch.start' },
]
</script>

<template>
  <div
    class="pointer-events-none fixed inset-0 touch-none select-none"
    :aria-label="t('touch.label')"
  >
    <template v-if="props.shoulders">
      <Button
        v-for="s in SHOULDERS"
        :key="s.button"
        :variant="ButtonVariant.Touch"
        :size="ButtonSize.TouchShoulder"
        class="pointer-events-auto absolute top-12"
        :class="s.place"
        @pointerdown="hold($event, s.button)"
        @pointerup="press(s.button, false)"
        @pointercancel="press(s.button, false)"
      >
        {{ t(s.key) }}
      </Button>
    </template>

    <Button
      :variant="ButtonVariant.Touch"
      :size="ButtonSize.TouchMenu"
      class="pointer-events-auto absolute top-12 left-1/2 -translate-x-1/2"
      :aria-label="t('touch.menu')"
      @click="emit('menu')"
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

    <div
      ref="dpad"
      class="pointer-events-auto absolute bottom-[64px] left-24 size-[148px] touch-none"
      role="group"
      :aria-label="t('touch.dpad')"
      @pointerdown="dpadDown"
      @pointermove="dpadMove"
      @pointerup="setDirs([])"
      @pointercancel="setDirs([])"
    >
      <span
        v-for="arm in ARMS"
        :key="arm.dir"
        class="absolute rounded-6 border border-line-strong"
        :class="[arm.place, dirs.includes(arm.dir) ? 'bg-surface-hover' : 'bg-glass']"
      />
    </div>

    <div class="absolute right-24 bottom-[64px] size-[148px]">
      <Button
        v-for="f in FACES"
        :key="f.button"
        :variant="ButtonVariant.Touch"
        :size="ButtonSize.TouchFace"
        class="pointer-events-auto absolute"
        :class="f.place"
        @pointerdown="hold($event, f.button)"
        @pointerup="press(f.button, false)"
        @pointercancel="press(f.button, false)"
      >
        {{ t(f.key) }}
      </Button>
    </div>

    <div class="absolute bottom-16 left-1/2 flex -translate-x-1/2 gap-12">
      <Button
        v-for="b in SYSTEM"
        :key="b.button"
        :variant="ButtonVariant.Touch"
        :size="ButtonSize.TouchSystem"
        class="pointer-events-auto"
        @pointerdown="hold($event, b.button)"
        @pointerup="press(b.button, false)"
        @pointercancel="press(b.button, false)"
      >
        {{ t(b.key) }}
      </Button>
    </div>
  </div>
</template>
