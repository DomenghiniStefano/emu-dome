<script setup lang="ts">
// A scan in progress (S03's bar, indeterminate: the total is not known up front), with the games
// found so far.
import { useI18n } from 'vue-i18n'
import { Progress } from '@/components/ui/progress'
import LibraryBanner from './LibraryBanner.vue'

defineProps<{ progress: { found: number } | null }>()

const { t } = useI18n()
</script>

<template>
  <LibraryBanner>
    <span class="font-medium whitespace-nowrap">{{ t('library.scan.title') }}</span>
    <Progress class="max-w-scan min-w-0 flex-1" :label="t('library.scan.title')" />
    <span v-if="progress" class="text-ink-muted tabular-nums">
      {{ t('library.scan.found', { found: progress.found }, progress.found) }}
    </span>
  </LibraryBanner>
</template>
