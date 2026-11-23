<script setup lang="ts">
// The library's toolbar on the phone (#29, owner 2026-11-22, mockup «Libreria sul telefono»): the
// console chips by their short names on one row that scrolls sideways; under it, from the
// content's edge, the order (L03, its name alone, as wide as the row leaves it), the Lista /
// Griglia switch (`library.view`, this device's) and «Aggiungi giochi» as a square «+». On its
// side, one row for both. No heading: the tab bar or the side column already says Libreria.
import { useI18n } from 'vue-i18n'
import { Plus } from '@lucide/vue'
import { Button, ButtonSize } from '@/design-system/components/button'
import { Segmented, SegmentedItem, SegmentedVariant } from '@/design-system/components/segmented'
import type { ConsoleId } from '@/emu'
import { GAMES_VIEWS, isGamesView, type ConsoleChip, type GamesView } from './libraryView'
import type { ShowKey, Sort } from './librarySort'
import ConsoleChips from './ConsoleChips.vue'
import SortFilterButton from './SortFilterButton.vue'

defineProps<{
  chips: ConsoleChip[]
  sort: Sort
  show: Record<ShowKey, boolean>
}>()
const consoleFilter = defineModel<ConsoleId | 'all'>('console', { required: true })
const view = defineModel<GamesView>('view', { required: true })
const emit = defineEmits<{ sort: [Sort]; toggle: [ShowKey]; reset: []; addFolder: [] }>()

const { t } = useI18n()

/** Segmented speaks strings: only the view's own ids are taken. */
function pick(v: string) {
  if (isGamesView(v)) view.value = v
}
</script>

<template>
  <!-- On its side the window is short and wide: the chips and the tools share one row. -->
  <div class="flex flex-col gap-8 landscape:flex-row landscape:items-center">
    <ConsoleChips v-model="consoleFilter" class="landscape:flex-1" :chips="chips" initial short />
    <div class="flex items-center gap-8">
      <SortFilterButton
        class="flex-1"
        short
        :sort="sort"
        :show="show"
        @sort="s => emit('sort', s)"
        @toggle="k => emit('toggle', k)"
        @reset="emit('reset')"
      />
      <Segmented
        :model-value="view"
        :variant="SegmentedVariant.Bar"
        :aria-label="t('settings.library.view.name')"
        @update:model-value="pick"
      >
        <SegmentedItem v-for="id in GAMES_VIEWS" :key="id" :value="id">
          {{ t(`settings.library.view.option.${id}`) }}
        </SegmentedItem>
      </Segmented>
      <Button
        :size="ButtonSize.ToolIcon"
        :icon="Plus"
        :label="t('library.addGames')"
        @click="emit('addFolder')"
      />
    </div>
  </div>
</template>
