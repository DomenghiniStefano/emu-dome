<script setup lang="ts">
// One row of C02's table: the game button, the device's button that gives it (a capture field:
// pressed, it listens for the new one) and what the row has to say (used twice, changed).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { CaptureField, CaptureFieldVariant } from '@/components/ui/capture-field'
import { RemapRowStatus } from './remapTable'

const props = defineProps<{
  /** The game button's name. */
  button: string
  /** The device's button or key that gives it, by name. */
  source: string
  status: RemapRowStatus
  /** Listening for the new button: the seconds left. */
  listening: boolean
  secondsLeft: number
  keyboard: boolean
  /** The field the table's nav scope focuses first. */
  navInitial?: boolean
}>()
const emit = defineEmits<{ capture: [] }>()
const { t } = useI18n()

const fieldText = computed(() => {
  if (!props.listening) return props.source
  const key = props.keyboard ? 'controls.pressKey' : 'controls.pressButton'
  return t(key, { n: props.secondsLeft })
})
const statusText = computed(() => {
  if (props.status === RemapRowStatus.UsedTwice) return t('controls.usedTwice')
  if (props.status === RemapRowStatus.Changed) return t('settingsUi.changed')
  return ''
})
</script>

<template>
  <div
    class="grid grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)_auto] items-center gap-12 border-t border-line px-16 py-8 data-listening:bg-violet-tint"
    :data-listening="listening ? '' : undefined"
  >
    <span class="font-semibold">{{ button }}</span>
    <CaptureField
      :variant="CaptureFieldVariant.Field"
      :listening="listening"
      class="justify-self-start"
      :data-nav-initial="navInitial ? '' : undefined"
      :aria-label="t('controls.changeButton', { button, source })"
      @click="emit('capture')"
    >
      {{ fieldText }}
    </CaptureField>
    <span
      class="text-caption leading-[18px] text-ink-muted data-warn:text-warning"
      :data-warn="status === RemapRowStatus.UsedTwice ? '' : undefined"
    >
      {{ statusText }}
    </span>
  </div>
</template>
