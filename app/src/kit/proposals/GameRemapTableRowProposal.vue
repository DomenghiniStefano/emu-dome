<script setup lang="ts">
// Kit proposal (not applied): controls/RemapTableRow.vue with a fixed status column (the width of
// a shortcut's key column, 150) instead of `auto`, so every row of the table, and its head, cuts
// the same columns whatever its status says. `head` draws the table's head row with the same
// columns (ControlsRemap.vue). Everything else as in the component.
import { CaptureField, CaptureFieldVariant } from '@/components/ui/capture-field'

defineProps<{
  button?: string
  source?: string
  status?: string
  /** The status in the warning colour («Usato anche da un altro»). */
  warn?: boolean
  head?: boolean
}>()
</script>

<template>
  <div
    v-if="head"
    class="grid grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)_var(--spacing-shortcut-key)] items-center gap-12 bg-surface-raised px-16 py-8 text-overline text-ink-muted"
  >
    <span>NEL GIOCO</span>
    <span>SUL CONTROLLER</span>
    <span></span>
  </div>
  <div
    v-else
    class="grid grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)_var(--spacing-shortcut-key)] items-center gap-12 border-t border-line px-16 py-8"
  >
    <span class="font-semibold">{{ button }}</span>
    <CaptureField :variant="CaptureFieldVariant.Field" class="justify-self-start">
      {{ source }}
    </CaptureField>
    <span class="text-caption-tall" :class="warn ? 'text-warning' : 'text-ink-muted'">{{
      status
    }}</span>
  </div>
</template>
