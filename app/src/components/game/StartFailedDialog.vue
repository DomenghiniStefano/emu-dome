<script setup lang="ts">
// G02: the game could not start, and why.
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { DialogTone } from '@/components/ui/dialog'
import FlowDialog from '../FlowDialog.vue'

defineProps<{ title: string; reason: string }>()
const emit = defineEmits<{ close: [] }>()
const { t } = useI18n()
</script>

<template>
  <FlowDialog
    :title="t('flow.startFailed', { title })"
    :tone="DialogTone.Danger"
    @back="emit('close')"
  >
    <p class="m-0">{{ reason }}</p>
    <template #actions>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        data-nav-initial
        @click="emit('close')"
      >
        {{ t('flow.gotIt') }}
      </Button>
    </template>
  </FlowDialog>
</template>
