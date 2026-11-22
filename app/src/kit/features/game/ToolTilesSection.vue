<script setup lang="ts">
// The quick menu's tools (G07 Strumenti): four on a one-screen game, six on NDS / 3DS; on ones
// (Muto, A schermo intero, Scambia) pressed, the ones that cannot be used disabled with the note.
import { useI18n } from 'vue-i18n'
import ToolTiles from '@/components/quick-menu/ToolTiles.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { KitGroup } from '@/kit/review'
import GameStage from './GameStage.vue'
import { TOOLS_DUAL, TOOLS_GBA } from './mock'

const { t } = useI18n()
</script>

<template>
  <KitSection title="Tool tiles" :group="KitGroup.Game">
    <KitState label="GBA: four columns, Muto on">
      <GameStage panel>
        <ToolTiles :tools="TOOLS_GBA" :three-columns="false" note="" />
      </GameStage>
    </KitState>
    <KitState label="NDS / 3DS: three columns, swap and full screen on, the lid off with its note">
      <GameStage panel>
        <ToolTiles :tools="TOOLS_DUAL" three-columns :note="t('qm.lidUnsupported')" />
      </GameStage>
    </KitState>
  </KitSection>
</template>
