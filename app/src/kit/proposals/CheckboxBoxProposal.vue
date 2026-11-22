<script setup lang="ts">
// Kit proposal (Checkbox): the box drawn once, with a tick when checked, instead of the span each
// screen writes by hand. Shown on the Kit page only; if the owner takes it, it becomes a part of
// components/ui/checkbox (on Reka's CheckboxIndicator) and the screens drop their own span.
import { Check } from '@lucide/vue'

defineProps<{ checked: boolean }>()
</script>

<template>
  <span
    class="inline-flex size-14 shrink-0 items-center justify-center rounded-4"
    :class="checked ? 'bg-accent-text' : 'border border-line-strong'"
    aria-hidden="true"
  >
    <!-- The stroke set on the icon itself: the Menu Button paints every svg inside it ink-muted. -->
    <Check v-if="checked" class="stroke-bg" :size="12" :stroke-width="3" />
  </span>
</template>
