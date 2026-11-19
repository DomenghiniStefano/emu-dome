<script setup lang="ts">
// G06: how to open the quick menu, at the top over the game, the first few games. It fades out
// (600 ms, a value waiting for Cowork: spec Decision 5).
import { useI18n } from 'vue-i18n'
import KeyCap from './KeyCap.vue'

defineProps<{ shown: boolean }>()
const { t } = useI18n()
</script>

<template>
  <Transition leave-active-class="transition-opacity duration-hint" leave-to-class="opacity-0">
    <div
      v-if="shown"
      class="glass-thick pointer-events-none fixed top-16 left-1/2 z-overlay flex -translate-x-1/2 items-center gap-6 rounded-10 px-14 py-8 text-ink-muted"
      role="status"
    >
      <KeyCap>{{ t('flow.hintEsc') }}</KeyCap>
      <span>{{ t('flow.hintOr') }}</span>
      <KeyCap>{{ t('flow.hintHome') }}</KeyCap>
      <span>{{ t('flow.hintText') }}</span>
    </div>
  </Transition>
</template>
