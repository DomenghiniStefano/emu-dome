<script setup lang="ts">
// G11: the confirmation of «Riavvia il gioco», in place of the menu.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { RefreshCw } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import PageTitle from './PageTitle.vue'
import { FocusMark } from './view'

const props = defineProps<{
  /** The phone's sheet: the touch size. */
  touch?: boolean
}>()
const emit = defineEmits<{ cancel: []; confirm: [] }>()

const { t } = useI18n()
const size = computed(() => (props.touch ? ButtonSize.MenuTouch : ButtonSize.Menu))
const initial = { [FocusMark.SubInitial]: '' }
</script>

<template>
  <PageTitle>{{ t('qm.restartTitle') }}</PageTitle>
  <p class="m-0 text-ink-muted">{{ t('qm.restartText') }}</p>
  <div class="flex justify-end gap-8 pt-4">
    <Button
      v-bind="initial"
      :variant="ButtonVariant.Soft"
      :size="size"
      class="min-w-0"
      @click="emit('cancel')"
    >
      {{ t('qm.cancel') }}
    </Button>
    <Button
      :variant="ButtonVariant.DangerOutline"
      :size="size"
      class="min-w-0"
      @click="emit('confirm')"
    >
      <RefreshCw :size="18" :stroke-width="1.75" aria-hidden="true" />
      {{ t('qm.restartConfirm') }}
    </Button>
  </div>
</template>
