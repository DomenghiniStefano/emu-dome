<script setup lang="ts">
// Choice cards with a preview: a single choice (`choice`, style `cards`), or presets that each
// apply a bundle and are never selected (`apply`). An option shown but not available yet is
// faded and says so.
import { useI18n } from 'vue-i18n'
import { Card, CardDescription, CardVariant } from '@/components/ui/card'
import { cardPreview } from './settingRowModel'
import type { RowOption } from './useSettingRow'

defineProps<{
  settingKey: string
  label: string
  options: RowOption[]
  /** Presets (`apply`): each card applies a bundle, nothing is ever selected. */
  apply: boolean
  /** The chosen option (a single choice). */
  value: string
  disabled: boolean
  /** The whole row has no backend yet (presets): every card says so on hover. */
  notYet: boolean
}>()
const emit = defineEmits<{ set: [string] }>()
const { t } = useI18n()

function cardDesc(o: RowOption): string {
  return o.notYet ? `${o.desc} ${t('settingsUi.notYet')}`.trim() : o.desc
}
</script>

<template>
  <div
    class="grid grid-cols-[repeat(auto-fill,minmax(var(--spacing-card-min),1fr))] gap-10"
    :class="{ 'opacity-locked': disabled }"
    :role="apply ? 'group' : 'radiogroup'"
    :aria-label="label"
  >
    <Card
      v-for="(o, i) in options"
      :key="o.id"
      as="button"
      :variant="CardVariant.Raised"
      :selected="!apply && o.id === value"
      class="gap-6 disabled:cursor-not-allowed"
      :class="{ 'opacity-locked': o.notYet }"
      :role="apply ? undefined : 'radio'"
      :aria-checked="apply ? undefined : o.id === value"
      :disabled="disabled || o.notYet"
      :title="o.notYet || (apply && notYet) ? t('settingsUi.notYet') : undefined"
      @click="!apply && emit('set', o.id)"
    >
      <span
        class="block h-thumb rounded-6 border border-line"
        aria-hidden="true"
        :style="{ background: cardPreview(settingKey, o.id, i) }"
      ></span>
      <span class="font-semibold">{{ o.text }}</span>
      <CardDescription v-if="o.desc || o.notYet">{{ cardDesc(o) }}</CardDescription>
    </Card>
  </div>
</template>
