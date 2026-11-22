<script setup lang="ts">
// The console chips over the grid: «Tutti» and each console present, with how many games each.
// `short` (the phone, #29): the consoles' short names (GBA, DS) on one row that scrolls sideways
// when it does not fit, instead of the long names wrapping.
import { useI18n } from 'vue-i18n'
import { Chip, ChipVariant } from '@/components/ui/chip'
import type { ConsoleId } from '@/emu'
import { cn } from '@/lib/cn'
import type { ConsoleChip } from './libraryView'

withDefaults(
  defineProps<{
    chips: ConsoleChip[]
    /** «Tutti» takes the view's first focus (no «Continua a giocare» above). */
    initial: boolean
    short?: boolean
  }>(),
  { short: false },
)
const chosen = defineModel<ConsoleId | 'all'>({ required: true })

const { t } = useI18n()
</script>

<template>
  <div
    :class="
      cn('flex gap-8', short ? 'min-w-0 overflow-x-auto [scrollbar-width:none]' : 'flex-wrap')
    "
    role="group"
    :aria-label="t('library.filterConsole')"
  >
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
        >{{
          c.id === 'all' ? t('library.all') : t(`${short ? 'consoleShort' : 'console'}.${c.id}`)
        }}
        <span class="opacity-soft tabular-nums">{{ c.count }}</span></span
      >
    </Chip>
  </div>
</template>
