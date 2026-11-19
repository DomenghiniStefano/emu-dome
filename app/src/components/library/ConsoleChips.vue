<script setup lang="ts">
// The console chips over the grid: «Tutti» and each console present, with how many games each.
import { useI18n } from 'vue-i18n'
import { Chip, ChipVariant } from '@/components/ui/chip'
import type { ConsoleId } from '../../emu'
import type { ConsoleChip } from './libraryView'

defineProps<{
  chips: ConsoleChip[]
  /** «Tutti» takes the view's first focus (no «Continua a giocare» above). */
  initial: boolean
}>()
const chosen = defineModel<ConsoleId | 'all'>({ required: true })

const { t } = useI18n()
</script>

<template>
  <div class="flex flex-wrap gap-8" role="group" :aria-label="t('library.filterConsole')">
    <Chip
      v-for="c in chips"
      :key="c.id"
      :variant="ChipVariant.Filter"
      :selected="c.id === chosen"
      :aria-pressed="c.id === chosen"
      :data-nav-initial="initial && c.id === 'all' ? '' : undefined"
      @click="chosen = c.id"
    >
      <!-- One run of text: the space before the count is the text's own, not the chip's gap. -->
      <span
        >{{ c.id === 'all' ? t('library.all') : t(`console.${c.id}`) }}
        <span class="opacity-soft tabular-nums">{{ c.count }}</span></span
      >
    </Chip>
  </div>
</template>
