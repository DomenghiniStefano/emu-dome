<script setup lang="ts">
// The library scan's progress (S02): no total is known while walking the folders, so a band
// sweeps the track while it runs, and the track is full when it is done.
import { useI18n } from 'vue-i18n'

defineProps<{ running: boolean }>()
const { t } = useI18n()
</script>

<template>
  <div
    class="h-6 overflow-hidden rounded-full bg-surface-raised"
    role="progressbar"
    :aria-label="t('firstRun.games.progress')"
    :aria-valuenow="running ? undefined : 100"
  >
    <span
      class="block h-full bg-accent-text"
      :class="running ? 'w-3/10 animate-scan-sweep' : 'w-full'"
    ></span>
  </div>
</template>
