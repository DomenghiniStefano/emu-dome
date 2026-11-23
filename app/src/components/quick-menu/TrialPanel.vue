<script setup lang="ts">
// The changes «in prova» (G16, G10 and the phone's G09): what they are, that closing the game
// drops them, and the three answers (undo, save for this game, save for the whole console).
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import TrialActions from './TrialActions.vue'

defineProps<{
  title: string
  consoleName: string
}>()
const emit = defineEmits<{ discard: []; commit: ['game' | 'console'] }>()

const { t } = useI18n()
</script>

<template>
  <div
    class="flex flex-col gap-8 rounded-card border border-accent-text bg-surface p-card-pad"
    role="status"
  >
    <div>
      <div class="font-semibold">{{ title }}</div>
      <div class="text-caption text-ink-muted">{{ t('qm.trialHint') }}</div>
    </div>
    <div class="grid grid-cols-[1fr_1fr] gap-8">
      <TrialActions @discard="emit('discard')" @commit-game="emit('commit', 'game')" />
    </div>
    <Button
      :variant="ButtonVariant.Link"
      :size="ButtonSize.Text"
      class="self-start underline"
      @click="emit('commit', 'console')"
    >
      {{ t('qm.saveConsole', { console: consoleName }) }}
    </Button>
  </div>
</template>
