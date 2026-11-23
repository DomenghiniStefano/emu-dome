<script setup lang="ts">
// G11: the confirmation of «Riavvia il gioco», in place of the menu. «Annulla», the quiet way out,
// stays words (#104), so the confirmation is the one with an icon.
import { useI18n } from 'vue-i18n'
import { RefreshCw } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { FocusMark } from './view'
import { Heading, HeadingVariant } from '@/design-system/components/heading'

const emit = defineEmits<{ cancel: []; confirm: [] }>()

const { t } = useI18n()
const initial = { [FocusMark.SubInitial]: '' }
</script>

<template>
  <Heading as="h2" :variant="HeadingVariant.PageTitle">{{ t('qm.restartTitle') }}</Heading>
  <p class="m-0 text-ink-muted">{{ t('qm.restartText') }}</p>
  <div class="flex justify-end gap-8 pt-4">
    <Button
      v-bind="initial"
      :variant="ButtonVariant.Soft"
      :size="ButtonSize.Menu"
      class="min-w-0"
      @click="emit('cancel')"
    >
      {{ t('qm.cancel') }}
    </Button>
    <Button
      :variant="ButtonVariant.DangerOutline"
      :size="ButtonSize.Menu"
      class="min-w-0"
      :icon="RefreshCw"
      @click="emit('confirm')"
    >
      {{ t('qm.restartConfirm') }}
    </Button>
  </div>
</template>
