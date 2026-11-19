<script setup lang="ts">
// A colour among a few (`colour`, `general.accent`): round swatches, the chosen one ringed.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { swatchKey } from './settingRowModel'

const props = defineProps<{
  settingKey: string
  label: string
  swatches: string[]
  /** The chosen colour; none chosen means the first. */
  value: string
  disabled: boolean
}>()
const emit = defineEmits<{ set: [string] }>()
const { t, te } = useI18n()

const chosen = computed(() => props.value || props.swatches[0])
function swatchName(hex: string): string {
  const k = swatchKey(props.settingKey, hex)
  return te(k) ? t(k) : hex
}
</script>

<template>
  <div
    class="flex gap-10"
    :class="{ 'opacity-locked': disabled }"
    role="radiogroup"
    :aria-label="label"
  >
    <Button
      v-for="c in swatches"
      :key="c"
      :variant="ButtonVariant.Swatch"
      :size="ButtonSize.IconSmall"
      role="radio"
      :aria-checked="chosen === c"
      :aria-label="swatchName(c)"
      :title="swatchName(c)"
      :style="{ background: c }"
      :disabled="disabled"
      @click="emit('set', c)"
    />
  </div>
</template>
