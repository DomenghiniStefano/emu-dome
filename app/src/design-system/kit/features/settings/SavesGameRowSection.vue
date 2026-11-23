<script setup lang="ts">
// One game in the Saves list (and in «Scegli il gioco»): small cover, title, what it has saved,
// an optional badge; `selected` for the game shown on the right.
import SavesGameRow from '@/components/saves/SavesGameRow.vue'
import { Badge, BadgeVariant } from '@/design-system/components/badge'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'
import { GAMES } from './mock'
</script>

<template>
  <KitSection title="Saves game row" :group="KitGroup.Settings">
    <KitState label="The list's column: selected, plain, long title, save set aside">
      <div class="flex w-game-list flex-col gap-6">
        <SavesGameRow :game="GAMES.short" detail="Gioco + 3 rapidi · 2,4 MB" numeric selected />
        <SavesGameRow :game="GAMES.gb" detail="Gioco · 32 KB" numeric />
        <SavesGameRow :game="GAMES.long" detail="Gioco + 8 rapidi · 41,7 MB" numeric />
        <SavesGameRow :game="GAMES.gb" detail="Gioco · 32 KB" numeric>
          <Badge :variant="BadgeVariant.Warning">Messo da parte</Badge>
        </SavesGameRow>
      </div>
    </KitState>
    <KitState label="«Scegli il gioco» dialog: the console as the line">
      <div class="flex w-game-list flex-col gap-6">
        <SavesGameRow :game="GAMES.short" detail="Game Boy Advance" />
      </div>
    </KitState>
  </KitSection>
</template>
