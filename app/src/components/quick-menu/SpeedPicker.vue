<script setup lang="ts">
// Speed on the main screen (G07): Normale, Avanti veloce, Rallentatore, each with the factor its
// setting gives; «Imposta le velocità» opens G16.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { SettingValue, Speed } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Segmented, SegmentedItem, SegmentedVariant } from '@/components/ui/segmented'
import { msg } from '@/settingsModel'
import SectionLabel from './SectionLabel.vue'
import { SpeedChoiceId, chosenSpeed, type SpeedChoice } from './speedChoices'

const props = defineProps<{
  choices: SpeedChoice[]
  /** The speed the game runs at now. */
  speed: Speed
  /** The options of `speed.ffval` and `speed.slow`, for the factor under each mode. */
  ffOption: SettingValue | null
  slowOption: SettingValue | null
  touch?: boolean
}>()
const emit = defineEmits<{ speed: [Speed]; setSpeeds: [Event] }>()

const { t } = useI18n()

const chosen = computed(() => chosenSpeed(props.choices, props.speed))

/** A mode's factor as its setting names it («4×»), '' while unknown. */
const optionText = (key: string, id: SettingValue | null) =>
  typeof id === 'string' ? t(msg.option(key, id)) : ''
const LABEL: Record<SpeedChoiceId, string> = {
  [SpeedChoiceId.Normal]: 'qm.normal',
  [SpeedChoiceId.FastForward]: 'qm.ff',
  [SpeedChoiceId.SlowMotion]: 'qm.slow',
}
function factor(id: SpeedChoiceId): string {
  switch (id) {
    case SpeedChoiceId.Normal:
      return t('speed.factor', { value: 1 })
    case SpeedChoiceId.FastForward:
      return optionText('speed.ffval', props.ffOption)
    case SpeedChoiceId.SlowMotion:
      return optionText('speed.slow', props.slowOption)
  }
}

function choose(id: string) {
  const choice = props.choices.find(c => c.id === id)
  if (choice) emit('speed', choice.speed)
}
</script>

<template>
  <section class="flex flex-col gap-10" :aria-label="t('qm.speedTitle')">
    <div class="flex items-baseline justify-between gap-8">
      <SectionLabel :touch="touch">{{ t('qm.speedTitle') }}</SectionLabel>
      <Button
        :variant="ButtonVariant.Link"
        :size="touch ? ButtonSize.TextTouch : ButtonSize.Text"
        @click="emit('setSpeeds', $event)"
      >
        {{ t('qm.setSpeeds') }}
      </Button>
    </div>
    <Segmented
      :model-value="chosen"
      :variant="touch ? SegmentedVariant.StackedTouch : SegmentedVariant.Stacked"
      :aria-label="t('qm.speedTitle')"
      @update:model-value="choose"
    >
      <SegmentedItem v-for="c in choices" :key="c.id" :value="c.id">
        <span class="font-semibold whitespace-nowrap" :class="{ 'text-(length:--text-label)': touch }">
          {{ t(LABEL[c.id]) }}
        </span>
        <span class="text-(length:--text-caption) font-medium tabular-nums opacity-[0.8]">
          {{ factor(c.id) }}
        </span>
      </SegmentedItem>
    </Segmented>
  </section>
</template>
