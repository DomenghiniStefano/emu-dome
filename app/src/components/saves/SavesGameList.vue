<script setup lang="ts">
// V01, the games that have saves: the title with «Importa…», the search, and one row per game
// (what it has saved; «Salvataggio illeggibile» when its in-game save was set aside, E04).
import { useI18n } from 'vue-i18n'
import type { GameId, LibraryGame } from '@/emu'
import { Button } from '@/components/ui/button'
import { SearchField } from '@/components/ui/search-field'
import SavesGameRow from './SavesGameRow.vue'
import { Heading } from '@/components/ui/heading'
import { Badge, BadgeVariant } from '@/components/ui/badge'

/** A game of the list with its line: what it has saved, and whether a save was set aside. */
export interface SavesListRow {
  game: LibraryGame
  detail: string
  broken: boolean
}

defineProps<{
  rows: SavesListRow[]
  /** The game shown on the right; null while the import is. */
  current: GameId | null
}>()
const query = defineModel<string>('query', { required: true })
const emit = defineEmits<{ pick: [LibraryGame]; import: [] }>()

const { t } = useI18n()
</script>

<template>
  <aside
    class="flex min-h-0 flex-none basis-game-list flex-col gap-14 overflow-auto border-r border-line px-16 py-24"
    :aria-label="t('saves.games')"
  >
    <div class="flex items-center justify-between gap-8">
      <Heading as="h1">{{ t('shell.saves') }}</Heading>
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
        :selected="game.id === current"
        :aria-current="game.id === current ? 'true' : undefined"
        :data-nav-initial="i === 0 ? '' : undefined"
        @click="emit('pick', game)"
      >
        <Badge v-if="broken" :variant="BadgeVariant.Warning">{{ t('saves.brokenBadge') }}</Badge>
      </SavesGameRow>
      <p v-if="rows.length === 0" class="m-0 text-caption text-ink-muted">
        {{ t('saves.noneShown') }}
      </p>
    </div>
  </aside>
</template>
