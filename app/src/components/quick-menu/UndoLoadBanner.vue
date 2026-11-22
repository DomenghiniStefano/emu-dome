<script setup lang="ts">
// G17 with a controller: after a load, «Annulla il caricamento» sits at the top of the menu until
// the next save or load.
import { useI18n } from 'vue-i18n'
import { RotateCcw } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardPad } from '@/components/ui/card'

defineProps<{
  /** What was loaded («Caricato dallo slot 2»). */
  text: string
}>()
defineEmits<{ undo: [] }>()

const { t } = useI18n()
</script>

<template>
  <Card :pad="CardPad.Tight" class="flex-row items-center gap-8 font-semibold" role="status">
    <span class="min-w-0 flex-1">{{ text }}</span>
    <Button
      :variant="ButtonVariant.Soft"
      :size="ButtonSize.MenuCompact"
      class="min-w-0"
      @click="$emit('undo')"
    >
      <RotateCcw :size="16" :stroke-width="1.75" aria-hidden="true" />
      {{ t('qm.undoLoad') }}
    </Button>
  </Card>
</template>
