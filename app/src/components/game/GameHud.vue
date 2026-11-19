<script setup lang="ts">
// In game there is no interface, except these discreet indicators at the top left: the speed
// when it is not 1×, «In pausa» (L3 + A, menu closed), and the FPS (L3 + Y, F10).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Speed } from '../../emu'
import { speedLabel } from '../../i18n/labels'
import { speedShown } from '../../lib/shell/session'
import HudChip from './HudChip.vue'

const props = defineProps<{
  speed: Speed
  /** Fast forward held (Tab, or the controller's turbo): `ffSpeed` is shown instead. */
  turboHeld: boolean
  ffSpeed: Speed
  paused: boolean
  /** The emulated FPS, when the counter is on; null: off. */
  fps: number | null
}>()
const { t, n } = useI18n()

const speedText = computed(() => speedLabel(props.turboHeld ? props.ffSpeed : props.speed))
</script>

<template>
  <div class="pointer-events-none fixed top-16 left-16 flex gap-8">
    <HudChip v-if="speedShown(turboHeld, speed)" class="tabular-nums">{{ speedText }}</HudChip>
    <HudChip v-if="paused">{{ t('hud.paused') }}</HudChip>
    <HudChip v-if="fps !== null" class="tabular-nums">
      {{ t('hud.fps', { fps: n(fps, { minimumFractionDigits: 1, maximumFractionDigits: 1 }) }) }}
    </HudChip>
  </div>
</template>
