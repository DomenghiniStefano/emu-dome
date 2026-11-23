<script setup lang="ts">
// `saves.resume` = ask: resume from the auto quick save, or start over. A game whose core cannot
// load states (Trello #93) keeps «Riprendi» disabled with the reason, and the focus on the way on.
import { useI18n } from 'vue-i18n'
import { Play, RotateCcw } from '@lucide/vue'
import type { StateLoadBlock } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { stateLoadBlockText } from '@/i18n/labels'
import FlowDialog from './FlowDialog.vue'

defineProps<{
  /** Why loading is off for this game (`GameInfo.stateLoadBlock`), else null. */
  loadBlock: StateLoadBlock | null
}>()
const emit = defineEmits<{ answer: [resume: boolean] }>()
const { t } = useI18n()
</script>

<template>
  <FlowDialog :title="t('flow.resumeTitle')" @back="emit('answer', false)">
    <p v-if="loadBlock !== null" class="m-0">{{ stateLoadBlockText(loadBlock) }}</p>
    <template #actions>
      <Button
        :size="ButtonSize.Medium"
        :icon="RotateCcw"
        :data-nav-initial="loadBlock !== null ? '' : undefined"
        @click="emit('answer', false)"
      >
        {{ t('flow.restart') }}
      </Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        :disabled="loadBlock !== null"
        :icon="Play"
        icon-filled
        :data-nav-initial="loadBlock !== null ? undefined : ''"
        @click="emit('answer', true)"
      >
        {{ t('flow.resume') }}
      </Button>
    </template>
  </FlowDialog>
</template>
