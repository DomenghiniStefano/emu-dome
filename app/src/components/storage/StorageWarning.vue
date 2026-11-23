<script setup lang="ts">
// E13 «Telefono quasi pieno» (Trello #29, the owner's option C): a warning at the top of the
// shell, «Spazio quasi finito: 380 MB liberi», with «Gestisci lo spazio», which opens that
// screen (StoragePage, in Settings › Salvataggi, through the shell's jump). The phone only (the
// shell mounts it there); the room is asked again whenever the shell shows (useStorageStatus.ts).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ArrowRight, HardDrive } from '@lucide/vue'
import { Button } from '@/design-system/components/button'
import { useShellJump } from '@/components/shell/useShellJump'
import { sizeLabel } from '@/i18n/labels'
import { useStorageStatus } from './useStorageStatus'

const { t } = useI18n()
const { status } = useStorageStatus()
// Mounted inside the shell, which provides the jump; elsewhere the button stays disabled.
const jump = useShellJump()

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
    <Button :icon="ArrowRight" :disabled="!jump" @click="jump?.('manageSpace')">
      {{ t('storage.manage') }}
    </Button>
  </div>
</template>
