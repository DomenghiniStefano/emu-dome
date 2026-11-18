<script setup lang="ts">
// «I tuoi giochi» over the grid: the console chips, the order and L03, «Aggiungi giochi».
import { useI18n } from 'vue-i18n'
import { Plus } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import type { ConsoleId } from '../../emu'
import type { ShowKey, Sort } from '../../librarySort'
import ConsoleChips from './ConsoleChips.vue'
import type { ConsoleChip } from './libraryView'
import SortFilterButton from './SortFilterButton.vue'

defineProps<{
  chips: ConsoleChip[]
  /** The chips take the view's first focus. */
  initialChip: boolean
  sort: Sort
  show: Record<ShowKey, boolean>
}>()
const consoleFilter = defineModel<ConsoleId | 'all'>('console', { required: true })
const emit = defineEmits<{ sort: [Sort]; toggle: [ShowKey]; reset: []; addFolder: [] }>()

const { t } = useI18n()
</script>

<template>
  <div class="flex flex-wrap items-center gap-12">
    <h2 class="m-0 mr-8 text-title">{{ t('library.yourGames') }}</h2>
    <ConsoleChips v-model="consoleFilter" :chips="chips" :initial="initialChip" />
    <div class="flex-1"></div>
    <SortFilterButton
      :sort="sort"
      :show="show"
      @sort="s => emit('sort', s)"
      @toggle="k => emit('toggle', k)"
      @reset="emit('reset')"
    />
    <Button @click="emit('addFolder')">
      <Plus :size="16" :stroke-width="1.75" aria-hidden="true" />
      {{ t('library.addGames') }}
    </Button>
  </div>
</template>
