<script setup lang="ts">
// The in-game save's backups (V02), newest first: when, why it was made, «Ripristina» (its icon
// alone, #104: repeated on every row).
import { useI18n } from 'vue-i18n'
import { ArchiveRestore } from '@lucide/vue'
import type { SaveBackup } from '@/emu'
import { saveWhenLabel } from '@/i18n/labels'
import { Button, ButtonSize } from '@/design-system/components/button'
import { Card, CardVariant } from '@/design-system/components/card'
import RowColumns from '@/components/settings/RowColumns.vue'

defineProps<{ backups: SaveBackup[] }>()
const emit = defineEmits<{ restore: [name: string] }>()

const { t } = useI18n()
</script>

<template>
  <Card :variant="CardVariant.List">
    <RowColumns>
      <div
        v-for="b in backups"
        :key="b.name"
        class="flex items-center gap-8 px-12 py-6 not-first:border-t not-first:border-line"
      >
        <span class="min-w-0 flex-1 tabular-nums">{{ saveWhenLabel(b.at) }}</span>
        <span class="text-caption text-ink-muted">{{ t(`saves.reason.${b.reason}`) }}</span>
        <Button
          :size="ButtonSize.ToolIcon"
          :icon="ArchiveRestore"
          :label="t('saves.restore')"
          @click="emit('restore', b.name)"
        />
      </div>
    </RowColumns>
  </Card>
</template>
