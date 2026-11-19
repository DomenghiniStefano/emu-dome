<script setup lang="ts">
// Files or folders dragged over the window, outside the game: they go into the library.
// The blur (8) and the title's tracking (none, unlike the title step) wait for Cowork (spec
// Decision 5).
import { useI18n } from 'vue-i18n'
import { Library } from '@lucide/vue'
import { Heading } from '@/components/ui/heading'

const { t } = useI18n()
</script>

<template>
  <div
    class="pointer-events-none fixed inset-0 z-overlay flex bg-scrim p-16 backdrop-blur-dim"
    aria-hidden="true"
  >
    <div
      class="flex flex-1 flex-col items-center justify-center gap-14 rounded-16 border-strong border-dashed border-accent-text text-accent-text"
    >
      <Library :size="44" :stroke-width="1.5" />
      <Heading as="div" class="text-ink">{{ t('library.drop') }}</Heading>
    </div>
  </div>
</template>
