<script setup lang="ts">
// Minimal touch overlay (Phase 0, Android): D-pad, A/B, L/R, Start/Select and the quick menu.
// Each control keeps its own pointer, so several can be held at once.
import { onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type Button } from '../emu'

const emit = defineEmits<{ menu: [] }>()
const { t } = useI18n()

/** Buttons currently held by the overlay, so nothing stays pressed when it goes away. */
const held = new Set<Button>()

function press(button: Button, pressed: boolean) {
  if (pressed === held.has(button)) return
  if (pressed) held.add(button)
  else held.delete(button)
  void emu.input(button, pressed)
}

function hold(e: PointerEvent, button: Button) {
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  press(button, true)
}

// D-pad: one area, the direction follows the finger (diagonals included).
const dpad = ref<HTMLElement | null>(null)
const dirs = ref<Button[]>([])
const DEADZONE = 0.25

function dpadMove(e: PointerEvent) {
  const el = dpad.value
  if (!el || !el.hasPointerCapture(e.pointerId)) return
  const r = el.getBoundingClientRect()
  const x = ((e.clientX - r.left) / r.width) * 2 - 1
  const y = ((e.clientY - r.top) / r.height) * 2 - 1
  const next: Button[] = []
  if (Math.hypot(x, y) > DEADZONE) {
    const angle = Math.atan2(y, x)
    const near = (target: number) =>
      Math.abs(Math.atan2(Math.sin(angle - target), Math.cos(angle - target))) < (Math.PI * 3) / 8
    if (near(0)) next.push('right')
    if (near(Math.PI)) next.push('left')
    if (near(Math.PI / 2)) next.push('down')
    if (near(-Math.PI / 2)) next.push('up')
  }
  setDirs(next)
}

function setDirs(next: Button[]) {
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

const faces: Array<{ button: Button; key: string }> = [
  { button: 'b', key: 'touch.b' },
  { button: 'a', key: 'touch.a' },
]
</script>

<template>
  <div class="touch" :aria-label="t('touch.label')">
    <button
      class="pad shoulder left"
      @pointerdown="hold($event, 'l')"
      @pointerup="press('l', false)"
      @pointercancel="press('l', false)"
    >
      {{ t('touch.l') }}
    </button>
    <button
      class="pad shoulder right"
      @pointerdown="hold($event, 'r')"
      @pointerup="press('r', false)"
      @pointercancel="press('r', false)"
    >
      {{ t('touch.r') }}
    </button>

    <button class="pad menu" :aria-label="t('touch.menu')" @click="emit('menu')">
      <svg
        viewBox="0 0 24 24"
        width="20"
        height="20"
        fill="none"
        stroke="var(--ink)"
        stroke-width="2"
        stroke-linecap="round"
        aria-hidden="true"
      >
        <line x1="4" y1="6" x2="20" y2="6" />
        <line x1="4" y1="12" x2="20" y2="12" />
        <line x1="4" y1="18" x2="20" y2="18" />
      </svg>
    </button>

    <div
      ref="dpad"
      class="dpad"
      role="group"
      :aria-label="t('touch.dpad')"
      @pointerdown="dpadDown"
      @pointermove="dpadMove"
      @pointerup="setDirs([])"
      @pointercancel="setDirs([])"
    >
      <span class="arm up" :class="{ on: dirs.includes('up') }" />
      <span class="arm down" :class="{ on: dirs.includes('down') }" />
      <span class="arm left" :class="{ on: dirs.includes('left') }" />
      <span class="arm right" :class="{ on: dirs.includes('right') }" />
    </div>

    <div class="faces">
      <button
        v-for="f in faces"
        :key="f.button"
        class="pad face"
        :class="f.button"
        @pointerdown="hold($event, f.button)"
        @pointerup="press(f.button, false)"
        @pointercancel="press(f.button, false)"
      >
        {{ t(f.key) }}
      </button>
    </div>

    <div class="system">
      <button
        class="pad small"
        @pointerdown="hold($event, 'select')"
        @pointerup="press('select', false)"
        @pointercancel="press('select', false)"
      >
        {{ t('touch.select') }}
      </button>
      <button
        class="pad small"
        @pointerdown="hold($event, 'start')"
        @pointerup="press('start', false)"
        @pointercancel="press('start', false)"
      >
        {{ t('touch.start') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
/* The overlay lets touches through except on the controls. */
.touch {
  position: fixed;
  inset: 0;
  pointer-events: none;
  user-select: none;
  -webkit-user-select: none;
  touch-action: none;
}
.touch > *,
.faces > *,
.system > * {
  pointer-events: auto;
}
.pad {
  border: 1px solid var(--line-strong);
  background: var(--glass);
  color: var(--ink);
  font-weight: 600;
  touch-action: none;
  -webkit-tap-highlight-color: transparent;
}
.pad:active {
  background: var(--surface-hover);
}
.shoulder {
  position: absolute;
  top: 12px;
  width: 88px;
  height: 40px;
  border-radius: var(--radius-10);
}
.shoulder.left {
  left: 12px;
}
.shoulder.right {
  right: 12px;
}
.menu {
  position: absolute;
  top: 12px;
  left: 50%;
  transform: translateX(-50%);
  width: 48px;
  height: 40px;
  border-radius: var(--radius-10);
  display: grid;
  place-items: center;
}
.dpad {
  position: absolute;
  left: 24px;
  bottom: 64px;
  width: 148px;
  height: 148px;
  touch-action: none;
}
.arm {
  position: absolute;
  background: var(--glass);
  border: 1px solid var(--line-strong);
  border-radius: var(--radius-6);
}
.arm.on {
  background: var(--surface-hover);
}
.arm.up,
.arm.down {
  left: 50px;
  width: 48px;
  height: 52px;
}
.arm.left,
.arm.right {
  top: 50px;
  width: 52px;
  height: 48px;
}
.arm.up {
  top: 0;
}
.arm.down {
  bottom: 0;
}
.arm.left {
  left: 0;
}
.arm.right {
  right: 0;
}
.faces {
  position: absolute;
  right: 24px;
  bottom: 64px;
  width: 148px;
  height: 148px;
  pointer-events: none;
}
.face {
  position: absolute;
  width: 64px;
  height: 64px;
  border-radius: 50%;
  font-size: 18px;
}
.face.a {
  right: 0;
  top: 16px;
}
.face.b {
  left: 0;
  bottom: 16px;
}
.system {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 12px;
  pointer-events: none;
}
.small {
  height: 32px;
  padding: 0 14px;
  border-radius: 16px;
  font-size: 11px;
  letter-spacing: 0.08em;
}
</style>
