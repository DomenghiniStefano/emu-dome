<script setup lang="ts">
// One game in a list of the Saves section: its small cover, the title and a line under it (what
// it has saved, or its console), and an optional badge (the slot). A button: the list and the
// «Scegli il gioco» dialog choose a game with it; `selected` marks the chosen one.
import type { LibraryGame } from '../../emu'
import { Card, CardVariant } from '@/components/ui/card'
import GameCover from '../GameCover.vue'

defineProps<{
  game: LibraryGame
  detail: string
  /** The detail is figures (sizes, counts): tabular digits. */
  numeric?: boolean
  selected?: boolean
}>()
</script>

<template>
  <Card :variant="CardVariant.ListItem" as="button" :selected="selected">
    <!-- GameCover draws itself with its own scoped CSS, which beats any utility: `!` makes it
         the 28px thumbnail of a row (TODO: a size of GameCover's own, Library stream). -->
    <GameCover
      class="w-[28px]! flex-none rounded-4! p-0! text-[0px]!"
      :game="game"
      :badge="false"
    />
    <span class="flex min-w-0 flex-1 flex-col gap-2">
      <span class="font-medium wrap-anywhere">{{ game.title }}</span>
      <span class="text-caption text-ink-muted" :class="{ 'tabular-nums': numeric }">{{
        detail
      }}</span>
    </span>
    <slot />
  </Card>
</template>
