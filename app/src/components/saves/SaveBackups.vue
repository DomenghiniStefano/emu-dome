<script setup lang="ts">
// The in-game save's backups (V02), newest first: when, why it was made, «Ripristina».
import { useI18n } from 'vue-i18n'
import type { SaveBackup } from '../../emu'
import { saveWhenLabel } from '../../i18n/labels'
import { Button } from '@/components/ui/button'

defineProps<{ backups: SaveBackup[] }>()
const emit = defineEmits<{ restore: [name: string] }>()

const { t } = useI18n()
</script>

<template>
  <div class="overflow-hidden rounded-10 border border-line bg-surface">
    <div
      v-for="b in backups"
      :key="b.name"
      class="flex items-center gap-12 px-16 py-10 not-first:border-t not-first:border-line"
    >
      <span class="min-w-0 flex-1 tabular-nums">{{ saveWhenLabel(b.at) }}</span>
      <span class="text-caption text-ink-muted">{{ t(`saves.reason.${b.reason}`) }}</span>
      <Button @click="emit('restore', b.name)">{{ t('saves.restore') }}</Button>
    </div>
  </div>
</template>
