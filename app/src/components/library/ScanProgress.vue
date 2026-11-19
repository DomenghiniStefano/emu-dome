<script setup lang="ts">
// A scan in progress (S03's bar, indeterminate: the total is not known up front), with the games
// found so far.
import { useI18n } from 'vue-i18n'

defineProps<{ progress: { found: number } | null }>()

const { t } = useI18n()
</script>

<template>
  <div class="flex items-center gap-12" role="status">
    <span class="font-medium whitespace-nowrap">{{ t('library.scan.title') }}</span>
    <!-- The bar's 320px is its own longest. -->
    <span
      class="h-6 max-w-[320px] flex-1 overflow-hidden rounded-full bg-surface-raised"
      aria-hidden="true"
    >
      <span class="animate-scan-sweep block h-full w-[30%] rounded-[inherit] bg-accent-text"></span>
    </span>
    <span v-if="progress" class="text-ink-muted tabular-nums">
      {{ t('library.scan.found', { found: progress.found }, progress.found) }}
    </span>
  </div>
</template>
