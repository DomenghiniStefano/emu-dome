<script setup lang="ts">
// E14 «Controller scollegato» (owner, 2026-11-22; the phone mockup's question 3, option A): the
// game is paused; «Continua con il tocco» (the main action) goes on with the touch controls,
// «Aspetta il controller» keeps it paused until a controller connects. It closes by itself when
// one does (useControllerLoss). On the game flow's dialog (FlowDialog); B is the safe exit
// («Aspetta»).
import { useI18n } from 'vue-i18n'
import FlowDialog from '@/components/game/FlowDialog.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

const emit = defineEmits<{ wait: []; touch: [] }>()
const { t } = useI18n()
</script>

<template>
  <FlowDialog :title="t('touch.lost.title')" @back="emit('wait')">
    <p class="m-0">{{ t('touch.lost.text') }}</p>
    <template #actions>
      <Button :size="ButtonSize.Medium" @click="emit('wait')">
        {{ t('touch.lost.wait') }}
      </Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        data-nav-initial
        @click="emit('touch')"
      >
        {{ t('touch.lost.touch') }}
      </Button>
    </template>
  </FlowDialog>
</template>
