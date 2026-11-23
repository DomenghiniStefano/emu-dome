<script setup lang="ts">
// Speed on the main screen (G07): Normale, Avanti veloce, Rallentatore, each with the factor its
// setting gives; «Imposta le velocità» opens G16.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { SettingValue, Speed } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Segmented, SegmentedItem, SegmentedVariant } from '@/design-system/components/segmented'
import { msg } from '@/lib/settingsModel'
import { SpeedChoiceId, chosenSpeed, type SpeedChoice } from './speedChoices'
import { LabelledBlock } from '@/design-system/components/labelled-block'
import { QuickMenuView, openerMark } from './view'

const props = defineProps<{
  choices: SpeedChoice[]
  /** The speed the game runs at now. */
  speed: Speed
  /** The options of `speed.ffval` and `speed.slow`, for the factor under each mode. */
  ffOption: SettingValue | null
  slowOption: SettingValue | null
}>()
const emit = defineEmits<{ speed: [Speed]; setSpeeds: [Event] }>()

const { t } = useI18n()
const setSpeedsMark = openerMark(QuickMenuView.Speed)

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
  <LabelledBlock :title="t('qm.speedTitle')" :aria-label="t('qm.speedTitle')">
    <template #aside>
      <Button
        v-bind="setSpeedsMark"
        :variant="ButtonVariant.Link"
        :size="ButtonSize.LabelLink"
        @click="emit('setSpeeds', $event)"
      >
        {{ t('qm.setSpeeds') }}
      </Button>
    </template>
    <Segmented
      :model-value="chosen"
      :variant="SegmentedVariant.Stacked"
      :aria-label="t('qm.speedTitle')"
      @update:model-value="choose"
    >
      <SegmentedItem v-for="c in choices" :key="c.id" :value="c.id">
        <span class="font-semibold whitespace-nowrap touch:text-label-inline">
          {{ t(LABEL[c.id]) }}
        </span>
        <span class="text-caption-inline font-medium tabular-nums opacity-soft">
          {{ factor(c.id) }}
        </span>
      </SegmentedItem>
    </Segmented>
  </LabelledBlock>
</template>
