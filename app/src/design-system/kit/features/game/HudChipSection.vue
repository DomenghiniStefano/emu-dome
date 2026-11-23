<script setup lang="ts">
// What shows over a running game: the HUD's chips at the top left (GameHud: the speed when not
// 1×, «In pausa», the FPS) and, the first few games, the hint at the top centre (QuickMenuHint).
// Both are `fixed` in the app; here they sit in a stage that holds them.
import HudChip from '@/components/game/HudChip.vue'
import QuickMenuHint from '@/components/game/QuickMenuHint.vue'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/design-system/kit/review'
import GameStage from './GameStage.vue'
</script>

<template>
  <KitSection title="HUD chip and quick menu hint" :group="KitGroup.Game">
    <KitState label="HUD: 4× · In pausa · 59.7 FPS">
      <GameStage :scrim="false">
        <div class="flex gap-8">
          <HudChip class="tabular-nums">4×</HudChip>
          <HudChip>In pausa</HudChip>
          <HudChip class="tabular-nums">59,7 FPS</HudChip>
        </div>
      </GameStage>
    </KitState>
    <KitState label="The hint, at the top centre of the game">
      <GameStage :scrim="false">
        <div class="h-cover-row w-panel max-w-full">
          <QuickMenuHint shown />
        </div>
      </GameStage>
    </KitState>

    <template #review>
      <KitIssue title="Tre vetri diversi in cima al gioco" :status="IssueStatus.Proposed">
        Sopra il gioco compaiono tre cose nella stessa fascia in alto: i chip dell’HUD, il
        suggerimento del menu rapido e i toast del gioco. Il suggerimento e i toast usano
        <code>glass-thick</code> (sfocatura, bordo luminoso, ombra); il chip dell’HUD ha solo il
        riempimento <code>bg-glass-thick</code>, senza sfocatura. Su un gioco chiaro o in movimento
        il chip si legge peggio e sembra di un’altra famiglia.
        <template #current>
          <GameStage :scrim="false">
            <div class="flex gap-8">
              <HudChip class="tabular-nums">4×</HudChip>
              <HudChip>In pausa</HudChip>
            </div>
          </GameStage>
        </template>
        <template #proposed>
          <GameStage :scrim="false">
            <div class="flex gap-8">
              <div
                class="flex h-control-sm items-center rounded-card px-8 font-semibold glass-thick"
              >
                4×
              </div>
              <div
                class="flex h-control-sm items-center rounded-card px-8 font-semibold glass-thick"
              >
                In pausa
              </div>
            </div>
          </GameStage>
        </template>
        <template #solution>
          in HudChip.vue sostituire <code>border border-line bg-glass-thick</code> con
          <code>glass-thick</code>, come QuickMenuHint e i toast del gioco. Se l’ombra del vetro è
          troppo per un indicatore discreto, un’utility <code>glass-chip</code> in
          <code>utilities.css</code> usata da tutti e tre.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
