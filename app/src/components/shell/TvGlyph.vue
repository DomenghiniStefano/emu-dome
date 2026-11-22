<script setup lang="ts">
// The button of the connected controller that carries out `action`, drawn as a key (TV mode,
// #30): the position comes from the routing and `pad.confirm`, its glyph from the controller's
// family (lib/padGlyphs.ts): a letter or a name, PlayStation's shapes as icons.
import { computed, type Component } from 'vue'
import { useI18n } from 'vue-i18n'
import { Circle, Square, Triangle, X } from '@lucide/vue'
import { Kbd, KbdSize, KbdVariant } from '@/components/ui/kbd'
import { uiMode } from '@/composables/useUiMode'
import { actionPos, padGlyph, type PadAction, type PadShape } from '@/lib/padGlyphs'

const props = withDefaults(
  defineProps<{
    action: PadAction
    /** On the main action's accent fill. */
    variant?: KbdVariant
  }>(),
  { variant: KbdVariant.Default },
)

const { t } = useI18n()

const SHAPES: Record<PadShape, Component> = {
  cross: X,
  circle: Circle,
  square: Square,
  triangle: Triangle,
}

const pos = computed(() => actionPos(props.action, uiMode.confirm))
const glyph = computed(() => padGlyph(uiMode.pad?.family ?? null, pos.value))
const shoulder = computed(() => pos.value === 'l1' || pos.value === 'r1')
</script>

<template>
  <Kbd :variant="variant" :size="shoulder ? KbdSize.TvShoulder : KbdSize.TvFace">
    <component
      :is="SHAPES[glyph.shape]"
      v-if="'shape' in glyph"
      :size="16"
      :stroke-width="2.25"
      :aria-label="t(`controls.pos.playstation.${pos}`)"
    />
    <template v-else>{{ t(glyph.key) }}</template>
  </Kbd>
</template>
