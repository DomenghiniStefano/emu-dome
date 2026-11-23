<script setup lang="ts">
// The phone's library (#29): a row of the list (library/GameRow.vue) and a tile of the compact
// grid (library/CompactTile.vue), in the touch density, about a phone's content wide.
// The game played last carries «Riprendi»; a family of regional versions its «N versioni» tag.
import CompactTile from '@/components/library/CompactTile.vue'
import GameRow from '@/components/library/GameRow.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'
import { GB_LONG, GBA, GBC_MISSING, N3DS_USER, NDS_ICON } from './mock'
</script>

<template>
  <KitSection title="Phone library: list row and compact tile" :group="KitGroup.Library">
    <KitState label="List (touch): «Riprendi», ROM icon with versions, long title, not reachable">
      <div data-density="touch" class="flex w-game-list max-w-full flex-col">
        <GameRow :game="GBA" resume />
        <GameRow :game="NDS_ICON" :versions="3" />
        <GameRow :game="N3DS_USER" />
        <GameRow :game="GB_LONG" />
        <GameRow :game="GBC_MISSING" />
      </div>
    </KitState>
    <KitState label="Compact grid (touch): «Riprendi» in the corner, ROM icon, versions">
      <div data-density="touch" class="grid w-game-list max-w-full grid-cols-3 gap-x-8 gap-y-12">
        <CompactTile :game="GBA" resume />
        <CompactTile :game="NDS_ICON" :versions="3" />
        <CompactTile :game="N3DS_USER" />
        <CompactTile :game="GB_LONG" />
        <CompactTile :game="GBC_MISSING" />
      </div>
    </KitState>
  </KitSection>
</template>
