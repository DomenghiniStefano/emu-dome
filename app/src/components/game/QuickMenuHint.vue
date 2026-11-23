<script setup lang="ts">
// G06: how to open the quick menu, at the top over the game, the first few games. It fades out
// (600 ms, a value waiting for a design decision: spec Decision 5).
import { useI18n } from 'vue-i18n'
import { Kbd } from '@/components/ui/kbd'

defineProps<{ shown: boolean }>()
const { t } = useI18n()
</script>

<template>
  <Transition leave-active-class="transition-opacity duration-hint" leave-to-class="opacity-0">
    <div
      v-if="shown"
      class="glass-thick pointer-events-none fixed top-12 left-1/2 z-overlay flex -translate-x-1/2 items-center gap-6 rounded-card px-10 py-6 text-ink-muted"
      role="status"
    >
      <Kbd>{{ t('flow.hintEsc') }}</Kbd>
      <span>{{ t('flow.hintOr') }}</span>
      <Kbd>{{ t('flow.hintHome') }}</Kbd>
      <span>{{ t('flow.hintText') }}</span>
    </div>
  </Transition>
</template>
