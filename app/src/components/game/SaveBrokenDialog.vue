<script setup lang="ts">
// E04: the game's in-game save cannot be read (the backend set it aside). With a backup: use it,
// or play without a save; without one: play without a save, or close.
// TODO(E04): «Apri la cartella» needs an opener for folders (no plugin yet).
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { DialogTone } from '@/components/ui/dialog'
import { dateTimeLabel } from '@/i18n/labels'
import FlowDialog from './FlowDialog.vue'

defineProps<{
  title: string
  /** Unix ms of the newest backup; null: there is none. */
  backupAt: number | null
}>()
const emit = defineEmits<{ use: []; play: []; close: [] }>()
const { t } = useI18n()
</script>

<template>
  <FlowDialog
    :title="t('flow.brokenTitle', { title })"
    :tone="DialogTone.Warning"
    @back="emit('close')"
  >
    <p v-if="backupAt !== null" class="m-0">
      {{ t('flow.brokenText', { game: title, date: dateTimeLabel(backupAt) }) }}
    </p>
    <p v-else class="m-0">{{ t('flow.brokenNoBackup', { game: title }) }}</p>
    <template #actions>
      <template v-if="backupAt !== null">
        <Button :size="ButtonSize.Medium" @click="emit('play')">{{ t('flow.brokenPlay') }}</Button>
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.Medium"
          data-nav-initial
          @click="emit('use')"
        >
          {{ t('flow.brokenUse') }}
        </Button>
      </template>
      <template v-else>
        <Button :size="ButtonSize.Medium" @click="emit('close')">{{ t('flow.close') }}</Button>
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.Medium"
          data-nav-initial
          @click="emit('play')"
        >
          {{ t('flow.brokenPlay') }}
        </Button>
      </template>
    </template>
  </FlowDialog>
</template>
