<script setup lang="ts">
// The app closing with a game open (#86, owner's choice B with D's icon): a glass strip, the save
// icon pulsing beside «Saving · don't turn off» while the game's saves are written, then a tick
// and «Saved». Only the look: ClosingOverlay.vue places it at the bottom of the window.
import { Check, Save } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

defineProps<{ saved: boolean }>()
const { t } = useI18n()
</script>

<template>
  <div
    class="glass-thick-flat flex items-center gap-12 rounded-card border border-line py-10 pr-18 pl-14 shadow-glass"
    role="status"
  >
    <Check v-if="saved" class="text-success" :size="20" :stroke-width="2" aria-hidden="true" />
    <!-- The pulse is on the container: the strokes fade together, never overlapping. -->
    <span v-else class="flex animate-save-pulse text-accent-text" aria-hidden="true">
      <Save :size="20" :stroke-width="2" />
    </span>
    <p class="m-0 flex items-baseline gap-4 whitespace-nowrap">
      <span class="text-label text-ink">{{
        saved ? t('closing.saved') : t('closing.saving')
      }}</span>
      <span v-if="!saved" class="text-body text-ink-muted">{{ t('closing.savingHint') }}</span>
    </p>
  </div>
</template>
