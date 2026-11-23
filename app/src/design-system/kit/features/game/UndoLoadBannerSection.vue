<script setup lang="ts">
// G17 with a controller: after a load, «Annulla il caricamento» at the top of the quick menu
// until the next save or load.
import UndoLoadBanner from '@/components/quick-menu/UndoLoadBanner.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'
import GameStage from './GameStage.vue'
</script>

<template>
  <KitSection title="Undo load banner" :group="KitGroup.Game">
    <KitState label="From a numbered slot">
      <GameStage panel>
        <UndoLoadBanner text="Caricato lo slot 2" />
      </GameStage>
    </KitState>
    <KitState label="From the auto slot">
      <GameStage panel>
        <UndoLoadBanner text="Caricato lo slot automatico" />
      </GameStage>
    </KitState>
  </KitSection>
</template>
