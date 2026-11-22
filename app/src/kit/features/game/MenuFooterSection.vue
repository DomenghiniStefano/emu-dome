<script setup lang="ts">
// The quick menu's foot: «Chiudi il gioco» far from «Riprendi il gioco», the keys that open the
// search between them, the resume key on the main action; per input in use.
import MenuFooter from '@/components/quick-menu/MenuFooter.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { KitGroup } from '@/kit/review'
import GameStage from './GameStage.vue'
</script>

<template>
  <KitSection title="Quick menu footer" :group="KitGroup.Game">
    <KitState label="Mouse and keyboard: Esc, Ctrl K">
      <GameStage panel>
        <MenuFooter hint="Esc" :search-keys="['Ctrl', 'K']" />
      </GameStage>
    </KitState>
    <KitState label="Controller: B, Y">
      <GameStage panel>
        <MenuFooter hint="B" :search-keys="['Y']" />
      </GameStage>
    </KitState>
    <KitState label="Touch layout (no hints; seen with the touch density on)">
      <GameStage panel>
        <MenuFooter :hint="null" touch />
      </GameStage>
    </KitState>
  </KitSection>
</template>
