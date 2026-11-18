<script setup lang="ts">
// E11: the emulator stopped. With an auto quick save, resuming from it is the main action;
// without one, back to the library is.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { DialogTone } from '@/components/ui/dialog'
import type { ConsoleId } from '../../emu'
import { timeAgo } from '../../lib/shell/timeAgo'
import FlowDialog from '../FlowDialog.vue'

const props = defineProps<{
  consoleId: ConsoleId
  /** Unix ms of the last auto quick save; null: none this session. */
  savedAt: number | null
}>()
const emit = defineEmits<{ copy: []; library: []; resume: [] }>()
const { t, locale } = useI18n()

const savedLine = computed(() =>
  props.savedAt
    ? t('flow.crashSaved', { when: timeAgo(props.savedAt, Date.now(), locale.value) })
    : t('flow.crashNoSave'),
)
</script>

<template>
  <FlowDialog :title="t('flow.crashTitle')" :tone="DialogTone.Danger" @back="emit('library')">
    <p class="m-0">
      {{ t('flow.crashText', { console: t(`console.${consoleId}`) }) }}
      {{ savedLine }}
    </p>
    <template #actions>
      <Button :size="ButtonSize.Medium" @click="emit('copy')">{{ t('flow.copyDetails') }}</Button>
      <Button
        :size="ButtonSize.Medium"
        :data-nav-initial="savedAt ? undefined : ''"
        @click="emit('library')"
      >
        {{ t('flow.backToLibrary') }}
      </Button>
      <Button
        v-if="savedAt"
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        data-nav-initial
        @click="emit('resume')"
      >
        {{ t('flow.resumeAuto') }}
      </Button>
    </template>
  </FlowDialog>
</template>
