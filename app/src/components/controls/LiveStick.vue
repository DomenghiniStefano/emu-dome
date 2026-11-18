<script setup lang="ts">
// C04's live stick: the dashed circle is the dead zone, the dot the stick's position now.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Card } from '@/components/ui/card'
import { deadZoneDiameter, stickDot } from './stick'

const props = defineProps<{
  /** The left stick, -1..1 each way (y down). */
  x: number
  y: number
  /** The dead zone, per cent of the travel (`pad.dead`). */
  dead: number
}>()
const { t } = useI18n()

/** The pad's side and the dot's, in px: the dot's position is computed from them. */
const SIZE = 120
const DOT = 12

const padStyle = { width: `${SIZE}px`, height: `${SIZE}px` }
const dotStyle = computed(() => {
  const { left, top } = stickDot(props.x, props.y, SIZE, DOT)
  return { left: `${left}px`, top: `${top}px` }
})
const deadStyle = computed(() => {
  const d = deadZoneDiameter(SIZE, props.dead)
  return { width: `${d}px`, height: `${d}px` }
})
</script>

<template>
  <Card class="flex-row items-center gap-24">
    <div
      class="relative flex-none rounded-full border border-line bg-surface-raised"
      role="img"
      :aria-label="t('controls.stickLabel')"
      :style="padStyle"
    >
      <span
        class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full border border-dashed border-line-strong"
        :style="deadStyle"
      ></span>
      <span class="absolute size-12 rounded-full bg-accent-text" :style="dotStyle"></span>
    </div>
    <div class="text-ink-muted">{{ t('controls.stickHint') }}</div>
  </Card>
</template>
