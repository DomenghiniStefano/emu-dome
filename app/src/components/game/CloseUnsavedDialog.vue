<script setup lang="ts">
// «Chiudere senza salvare?»: the quick menu's «Chiudi il gioco» with progress not in a quick save
// (closeAsk.ts, useGameEnding). «Salva e chiudi» (the accent, the focus) saves into the game's
// quick slot and closes once written; «Chiudi senza salvare» closes as before; «Annulla», the
// quiet way out, stays words (#104) and is B / back too.
import { useI18n } from 'vue-i18n'
import { Power, Save } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { CloseAnswer } from './closeAsk'
import FlowDialog from './FlowDialog.vue'

const props = defineProps<{
  /** `saves.autoexit`: the auto quick save keeps the latest progress. */
  autosave: boolean
  /** «Salva e chiudi» is waiting for its quick save: no other answer meanwhile. */
  saving: boolean
}>()
const emit = defineEmits<{ answer: [CloseAnswer] }>()
const { t } = useI18n()

const back = () => props.saving || emit('answer', CloseAnswer.Cancel)
</script>

<template>
  <FlowDialog :title="t('flow.unsaved.title')" @back="back">
    <p class="m-0">{{ autosave ? t('flow.unsaved.textAuto') : t('flow.unsaved.textLost') }}</p>
    <template #actions>
      <Button
        :variant="ButtonVariant.Soft"
        :size="ButtonSize.Medium"
        :disabled="saving"
        @click="emit('answer', CloseAnswer.Cancel)"
      >
        {{ t('flow.unsaved.cancel') }}
      </Button>
      <Button
        :variant="ButtonVariant.Danger"
        :size="ButtonSize.Medium"
        :icon="Power"
        :disabled="saving"
        @click="emit('answer', CloseAnswer.Discard)"
      >
        {{ t('flow.unsaved.discard') }}
      </Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        :icon="Save"
        :disabled="saving"
        data-nav-initial
        @click="emit('answer', CloseAnswer.Save)"
      >
        {{ t('flow.unsaved.save') }}
      </Button>
    </template>
  </FlowDialog>
</template>
