<script setup lang="ts">
// Every deletion of «Gestisci lo spazio» asks first (the owner's card), whatever
// `general.confirmdel` says: what goes and what stays («I salvataggi restano»). The safe way out
// takes the focus; the deletion is the danger action.
import type { Component } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { DialogTone } from '@/design-system/components/dialog'
import FlowDialog from '@/components/game/FlowDialog.vue'

defineProps<{ title: string; text: string; confirm: string; icon: Component }>()
const emit = defineEmits<{ confirm: []; cancel: [] }>()
const { t } = useI18n()
</script>

<template>
  <FlowDialog :title="title" :tone="DialogTone.Danger" escape-closes @back="emit('cancel')">
    <p class="m-0">{{ text }}</p>
    <template #actions>
      <Button :size="ButtonSize.Medium" data-nav-initial @click="emit('cancel')">
        {{ t('storage.page.cancel') }}
      </Button>
      <Button
        :variant="ButtonVariant.Danger"
        :size="ButtonSize.Medium"
        :icon="icon"
        @click="emit('confirm')"
      >
        {{ confirm }}
      </Button>
    </template>
  </FlowDialog>
</template>
