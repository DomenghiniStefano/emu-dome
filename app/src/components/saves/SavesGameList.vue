<script setup lang="ts">
// V01, the games that have saves: the title with «Importa…», the search, and one row per game
// (what it has saved; «Salvataggio illeggibile» when its in-game save was set aside, E04).
import { useI18n } from 'vue-i18n'
import type { LibraryGame } from '../../emu'
import { Button } from '@/components/ui/button'
import { SearchField } from '@/components/ui/search-field'
import SaveBadge from './SaveBadge.vue'
import SavesGameRow from './SavesGameRow.vue'
import { SaveBadgeTone } from './variants'

/** A game of the list with its line: what it has saved, and whether a save was set aside. */
export interface SavesListRow {
  game: LibraryGame
  detail: string
  broken: boolean
}

defineProps<{
  rows: SavesListRow[]
  /** ROM path of the game shown on the right; null while the import is. */
  currentPath: string | null
}>()
const query = defineModel<string>('query', { required: true })
const emit = defineEmits<{ pick: [LibraryGame]; import: [] }>()

const { t } = useI18n()
</script>

<template>
  <aside
    class="flex min-h-0 flex-[0_0_360px] flex-col gap-14 overflow-auto border-r border-line px-16 py-24"
    :aria-label="t('saves.games')"
  >
    <div class="flex items-center justify-between gap-8">
      <h1 class="m-0 text-title">{{ t('shell.saves') }}</h1>
      <Button @click="emit('import')">{{ t('saves.import') }}</Button>
    </div>
    <SearchField v-model="query" :label="t('saves.search')" :placeholder="t('saves.search')" />
    <div class="flex flex-col gap-2">
      <SavesGameRow
        v-for="({ game, detail, broken }, i) in rows"
        :key="game.id"
        :game="game"
        :detail="detail"
        numeric
        :selected="game.path === currentPath"
        :aria-current="game.path === currentPath ? 'true' : undefined"
        :data-nav-initial="i === 0 ? '' : undefined"
        @click="emit('pick', game)"
      >
        <SaveBadge v-if="broken" :tone="SaveBadgeTone.Warning">{{
          t('saves.brokenBadge')
        }}</SaveBadge>
      </SavesGameRow>
      <p v-if="rows.length === 0" class="m-0 text-caption text-ink-muted">
        {{ t('saves.noneShown') }}
      </p>
    </div>
  </aside>
</template>
