<script setup lang="ts">
// G09 quick picks over the cards: image size, smoothing and the screen effect in one press.
// A pick that would change a setting locked here is disabled; the setting's card says why.
import { useI18n } from 'vue-i18n'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import type { ScreenPick, ScreenPickId } from './screenPicks'

defineProps<{
  picks: ScreenPick[]
}>()
const emit = defineEmits<{ pick: [ScreenPickId] }>()

const { t } = useI18n()
</script>

<template>
  <div
    class="grid grid-cols-4 gap-8 touch:grid-cols-[1fr_1fr]"
    role="group"
    :aria-label="t('qm.picks.label')"
  >
    <Button
      v-for="p in picks"
      :key="p.id"
      :variant="ButtonVariant.Pick"
      :size="ButtonSize.MenuCompact"
      :fade="ButtonFade.Dim"
      class="min-w-0"
      :aria-pressed="p.on"
      :disabled="!p.available"
      @click="emit('pick', p.id)"
    >
      <span class="truncate">{{ t(p.label) }}</span>
    </Button>
  </div>
</template>
