<script setup lang="ts">
// The Google Drive space of the status card (Trello #31): «X liberi di Y», a bar with EmuDome's
// files and the account's other files, and their sizes. No limit: no bar; not measured yet: the
// line says so.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CloudStatus } from '@/emu'
import { sizeLabel } from '@/i18n/labels'
import { spaceBar } from './cloud'

const props = defineProps<{ status: CloudStatus }>()
const { t } = useI18n()

const bar = computed(() => spaceBar(props.status))
const line = computed(() => {
  const b = bar.value
  if (!b) return t('cloud.space.unknown')
  if (b.limit === null || b.free === null) return t('cloud.space.unlimited')
  return t('cloud.space.free', { free: sizeLabel(b.free), total: sizeLabel(b.limit) })
})
const percent = (share: number) => `${(share * 100).toFixed(2)}%`
</script>

<template>
  <div class="flex min-w-0 flex-col gap-6">
    <span class="text-caption text-ink-muted">{{ line }}</span>
    <div
      v-if="bar && bar.limit !== null"
      class="flex h-8 overflow-hidden rounded-full bg-surface-raised"
      role="img"
      :aria-label="line"
    >
      <span class="block h-full bg-accent" :style="{ width: percent(bar.emudome) }"></span>
      <span class="block h-full bg-line-strong" :style="{ width: percent(bar.other) }"></span>
    </div>
    <span v-if="bar" class="text-caption text-ink-muted tabular-nums">{{
      t('cloud.space.parts', {
        emudome: sizeLabel(bar.emudomeBytes),
        other: sizeLabel(bar.otherBytes),
      })
    }}</span>
  </div>
</template>
