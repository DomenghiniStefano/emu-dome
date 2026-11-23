<script setup lang="ts">
// C02's toolbar: «Si applica a» (the level the table is edited at, shown with more than one) and
// «Disposizione A/B» (`pad.ab`, controllers only).
import { useI18n } from 'vue-i18n'
import { Segmented, SegmentedItem, SegmentedVariant } from '@/design-system/components/segmented'
import type { ConsoleId } from '@/emu'
import { levelName, type RemapLevel } from './levels'

defineProps<{
  levels: RemapLevel[]
  /** The console in context: «Giochi <console>». */
  contextConsole: ConsoleId | null
  /** Show «Disposizione A/B». */
  withAb: boolean
}>()
const level = defineModel<RemapLevel>('level', { required: true })
const ab = defineModel<string>('ab', { required: true })
const { t } = useI18n()

const AB_LAYOUTS = ['nintendo', 'printed'] as const

/** Segmented hands a string: one of `levels`. */
function chooseLevel(value: string) {
  level.value = value as RemapLevel
}
</script>

<template>
  <div class="flex flex-wrap items-center justify-between gap-8">
    <div v-if="levels.length > 1" class="flex items-center gap-8">
      <span class="font-medium text-ink-muted">{{ t('controls.appliesTo') }}</span>
      <Segmented
        :model-value="level"
        :variant="SegmentedVariant.Strong"
        :aria-label="t('controls.appliesTo')"
        @update:model-value="chooseLevel"
      >
        <SegmentedItem v-for="l in levels" :key="l" :value="l">
          {{ levelName(l, contextConsole) }}
        </SegmentedItem>
      </Segmented>
    </div>
    <div v-if="withAb" class="flex items-center gap-8">
      <span class="font-medium text-ink-muted">{{ t('settings.pad.ab.name') }}</span>
      <Segmented
        v-model="ab"
        :variant="SegmentedVariant.Strong"
        :aria-label="t('settings.pad.ab.name')"
      >
        <SegmentedItem v-for="id in AB_LAYOUTS" :key="id" :value="id">
          {{ t(`controls.ab.${id}`) }}
        </SegmentedItem>
      </Segmented>
    </div>
  </div>
</template>
