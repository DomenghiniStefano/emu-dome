<script setup lang="ts">
// «Continua a giocare»: the games played last, whatever the filters.
import { useI18n } from 'vue-i18n'
import type { LibraryGame } from '@/emu'
import type { ResumePoint } from '@/lib/gameDetails'
import ContinueCard from './ContinueCard.vue'
import { Heading } from '@/components/ui/heading'

defineProps<{ games: LibraryGame[]; resume: Record<string, ResumePoint> }>()
const emit = defineEmits<{
  resume: [LibraryGame]
  openSaves: [LibraryGame]
  /** The pointer or the focus is on this game: its cover tints the haze. */
  foreground: [LibraryGame]
}>()

const { t } = useI18n()
</script>

<template>
  <section class="flex flex-col gap-8" :aria-label="t('library.continue.title')">
    <div class="flex flex-wrap items-baseline justify-between gap-8">
      <Heading as="h1">{{ t('library.continue.title') }}</Heading>
      <span class="text-caption text-ink-muted">{{ t('library.continue.lead') }}</span>
    </div>
    <div class="grid grid-cols-[repeat(auto-fill,minmax(var(--spacing-tile-min),1fr))] gap-8">
      <ContinueCard
        v-for="(g, i) in games"
        :key="g.id"
        :game="g"
        :point="resume[g.id]"
        :main="i === 0"
        @mouseenter="emit('foreground', g)"
        @focusin="emit('foreground', g)"
        @resume="emit('resume', g)"
        @open-saves="emit('openSaves', g)"
      />
    </div>
  </section>
</template>
