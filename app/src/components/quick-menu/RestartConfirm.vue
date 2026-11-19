<script setup lang="ts">
// G11: the confirmation of «Riavvia il gioco», in place of the menu.
import { useI18n } from 'vue-i18n'
import { RefreshCw } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { FocusMark } from './view'
import { Heading, HeadingVariant } from '@/components/ui/heading'

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
      @click="emit('confirm')"
    >
      <RefreshCw :size="18" :stroke-width="1.75" aria-hidden="true" />
      {{ t('qm.restartConfirm') }}
    </Button>
  </div>
</template>
