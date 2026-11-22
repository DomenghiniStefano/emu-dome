<script setup lang="ts">
// E13 «Telefono quasi pieno» (Trello #29, the owner's option C): a warning at the top of the
// shell, «Spazio quasi finito: 380 MB liberi», with «Gestisci lo spazio». That button's screen
// (the games by size, «Togli dal telefono») is not designed yet: the button shows faded, and a
// press says it arrives in a future update. The phone only (the shell mounts it there); the
// room is asked again whenever the shell shows (useStorageStatus.ts).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { HardDrive } from '@lucide/vue'
import { Button, ButtonFade } from '@/components/ui/button'
import { useToasts } from '@/composables/useToasts'
import { sizeLabel } from '@/i18n/labels'
import { useStorageStatus } from './useStorageStatus'

const { t } = useI18n()
const toasts = useToasts()
const { status } = useStorageStatus()

const free = computed(() => (status.value?.low ? sizeLabel(status.value.free) : null))
</script>

<template>
  <div
    v-if="free"
    class="mx-12 mt-8 flex flex-none items-center gap-8 rounded-card border border-warning-edge bg-warning-tint px-10 py-6"
    role="status"
  >
    <HardDrive class="flex-none text-warning" :size="18" :stroke-width="1.75" aria-hidden="true" />
    <span class="min-w-0 flex-1">{{ t('storage.low', { free }) }}</span>
    <Button
      :fade="ButtonFade.Dim"
      :icon="HardDrive"
      aria-disabled="true"
      @click="toasts.notify(t('storage.manageNotYet'))"
    >
      {{ t('storage.manage') }}
    </Button>
  </div>
</template>
