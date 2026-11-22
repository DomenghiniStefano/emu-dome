<script setup lang="ts">
// The app closing with a game open (#86, owner's choice B with D's icon): a glass strip, the save
// icon pulsing beside «Saving · don't turn off» while the game's saves are written, then a tick
// and «Saved». Only the look: ClosingOverlay.vue places it at the bottom of the window.
// The same strip tells the copy of a closed game's saves to Google Drive (G14, #31,
// cloud/CloudPushStrip.vue), with its own icon and words.
import type { Component } from 'vue'
import { computed } from 'vue'
import { Check, Save } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    saved: boolean
    /** The pulsing icon while the work runs (the save icon by default). */
    icon?: Component
    /** The words while it runs, its hint, and once done (the closing's by default). */
    busy?: string
    hint?: string
    done?: string
  }>(),
  { icon: () => Save, busy: undefined, hint: undefined, done: undefined },
)
const { t } = useI18n()

const busyText = computed(() => props.busy ?? t('closing.saving'))
const hintText = computed(() => props.hint ?? t('closing.savingHint'))
const doneText = computed(() => props.done ?? t('closing.saved'))
</script>

<template>
  <div
    class="glass-thick-flat flex items-center gap-8 rounded-card border border-line py-6 pr-12 pl-10 shadow-glass"
    role="status"
  >
    <Check v-if="saved" class="text-success" :size="20" :stroke-width="2" aria-hidden="true" />
    <!-- The pulse is on the container: the strokes fade together, never overlapping. -->
    <span v-else class="flex animate-save-pulse text-accent-text" aria-hidden="true">
      <component :is="icon" :size="20" :stroke-width="2" />
    </span>
    <p class="m-0 flex items-baseline gap-4 whitespace-nowrap">
      <span class="text-label text-ink">{{ saved ? doneText : busyText }}</span>
      <span v-if="!saved" class="text-body text-ink-muted">{{ hintText }}</span>
    </p>
  </div>
</template>
