<script setup lang="ts">
// E12: the game runs slower than normal; a lighter setting is suggested.
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize } from '@/components/ui/button'
import { DialogTone } from '@/components/ui/dialog'
import FlowDialog from './FlowDialog.vue'

defineProps<{ fps: number; target: number }>()
const emit = defineEmits<{ close: [] }>()
const { t } = useI18n()
</script>

<template>
  <FlowDialog
    :title="t('flow.slowTitle', { fps: Math.round(fps), target: Math.round(target) })"
    :tone="DialogTone.Warning"
    @back="emit('close')"
  >
    <p class="m-0">{{ t('flow.slowText') }}</p>
    <template #actions>
      <Button :size="ButtonSize.Medium" data-nav-initial @click="emit('close')">
        {{ t('flow.notNow') }}
      </Button>
    </template>
  </FlowDialog>
</template>
