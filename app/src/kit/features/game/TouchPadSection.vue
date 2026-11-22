<script setup lang="ts">
// The touch controls' two styles (`touch.tskin`, owner 2026-11-22) as the game shows them: every
// control of a 3DS (the most there are) through TouchControl, over a game's colours, at the
// style's fade. Playable here: press the d-pad between two arms, drag the Circle Pad, hold a
// button (each lights while held). Each control sits in a box of its own size.
import type { TouchControl as Control, TouchControlId } from '@/emu'
import TouchControl from '@/components/touch/TouchControl.vue'
import { TOUCH_SKINS } from '@/components/touch/skins'
import GameStage from '@/kit/features/game/GameStage.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { KitGroup } from '@/kit/review'

/** Each control and the box it is drawn in (its own size at 100 %). */
const CONTROLS: [TouchControlId, string][] = [
  ['l', 'h-pad-strip w-pad-shoulder'],
  ['menu', 'size-pad-strip'],
  ['r', 'h-pad-strip w-pad-shoulder'],
  ['dpad', 'size-dpad'],
  ['circlePad', 'size-pad-stick'],
  ['y', 'size-pad-face'],
  ['x', 'size-pad-face'],
  ['b', 'size-pad-face'],
  ['a', 'size-pad-face'],
  ['select', 'h-pad-system-h w-pad-system'],
  ['start', 'h-pad-system-h w-pad-system'],
]
const control = (id: TouchControlId): Control => ({
  id,
  x: 0.5,
  y: 0.5,
  size: 100,
  opacity: 100,
  behaviour: 'tap',
})
</script>

<template>
  <KitSection title="Touch controls: «Fumé» and «Contorno»" :group="KitGroup.Game">
    <KitState
      v-for="skin in TOUCH_SKINS"
      :key="skin"
      :label="`${skin}: L, menu, R, d-pad, Circle Pad, Y X B A, Select, Start (press them)`"
    >
      <GameStage :scrim="false">
        <div class="flex flex-wrap items-center gap-12">
          <div v-for="[id, box] in CONTROLS" :key="id" class="relative" :class="box">
            <TouchControl :control="control(id)" :skin="skin" :opacity="100" interactive />
          </div>
        </div>
      </GameStage>
    </KitState>
  </KitSection>
</template>
