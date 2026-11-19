<script setup lang="ts">
// One game in a list of the Saves section: its small cover, the title and a line under it (what
// it has saved, or its console), and an optional badge (the slot). A button: the list and the
// «Scegli il gioco» dialog choose a game with it; `selected` marks the chosen one.
import type { LibraryGame } from '@/emu'
import { Card, CardVariant } from '@/components/ui/card'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'

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
    <GameCover
      :size="CoverSize.Thumb"
      class="flex-none"
      :game="game"
      :badge="false"
      :title="false"
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
