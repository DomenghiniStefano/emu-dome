<script setup lang="ts">
// `saves.resume` = ask: resume from the auto quick save, or start over.
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import FlowDialog from '../FlowDialog.vue'

const emit = defineEmits<{ answer: [resume: boolean] }>()
const { t } = useI18n()
</script>

<template>
  <FlowDialog :title="t('flow.resumeTitle')" @back="emit('answer', false)">
    <template #actions>
      <Button :size="ButtonSize.Medium" @click="emit('answer', false)">
        {{ t('flow.restart') }}
      </Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        data-nav-initial
        @click="emit('answer', true)"
      >
        {{ t('flow.resume') }}
      </Button>
    </template>
  </FlowDialog>
</template>
