<script setup lang="ts">
// The phone quick menu's line after a save made in it (owner, 2026-11-22): the frame just saved,
// «Salvato nello slot N» and «Annulla» (the slot gets back what it held). It stays while the
// menu is open; resuming closes it.
import { useI18n } from 'vue-i18n'
import { Check, Undo2 } from '@lucide/vue'
import SaveThumb from '@/components/saves/SaveThumb.vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Card, CardPad } from '@/design-system/components/card'

defineProps<{
  /** The slot saved to. */
  saved: number
  /** The saved frame, once the slots are read again; null until then. */
  thumb: string | null
}>()
defineEmits<{ undo: [] }>()

const { t } = useI18n()
</script>

<template>
  <Card :pad="CardPad.Tight" class="flex-row items-center gap-8" role="status">
    <SaveThumb :src="thumb" crop class="aspect-4/3 w-thumb-sm shrink-0 rounded-badge" />
    <span class="flex min-w-0 flex-1 items-center gap-4 text-label font-semibold">
      <Check :size="16" :stroke-width="1.75" class="shrink-0 text-success" aria-hidden="true" />
      <span class="min-w-0">{{ t('qm.saved', { slot: saved }) }}</span>
    </span>
    <Button
      :variant="ButtonVariant.Soft"
      :size="ButtonSize.Tool"
      :icon="Undo2"
      @click="$emit('undo')"
    >
      {{ t('qm.undoSave') }}
    </Button>
  </Card>
</template>
