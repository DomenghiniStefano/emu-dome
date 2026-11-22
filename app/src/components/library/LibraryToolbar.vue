<script setup lang="ts">
// «I tuoi giochi» over the grid: the console chips, the order and L03, «Aggiungi giochi».
import { useI18n } from 'vue-i18n'
import { Plus } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import type { ConsoleId } from '@/emu'
import type { ShowKey, Sort } from './librarySort'
import ConsoleChips from './ConsoleChips.vue'
import type { ConsoleChip } from './libraryView'
import SortFilterButton from './SortFilterButton.vue'
import { Heading } from '@/components/ui/heading'

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
  <div class="flex flex-wrap items-center gap-8">
    <Heading class="mr-8">{{ t('library.yourGames') }}</Heading>
    <ConsoleChips v-model="consoleFilter" :chips="chips" :initial="initialChip" />
    <!-- The sort and «Aggiungi giochi» stay together at the row's end: in a narrow window they
         go under the chips as one group (on a phone, one under the other). -->
    <div class="ml-auto flex min-w-0 flex-wrap items-center gap-8">
      <SortFilterButton
        :sort="sort"
        :show="show"
        @sort="s => emit('sort', s)"
        @toggle="k => emit('toggle', k)"
        @reset="emit('reset')"
      />
      <Button :icon="Plus" @click="emit('addFolder')">
        {{ t('library.addGames') }}
      </Button>
    </div>
  </div>
</template>
