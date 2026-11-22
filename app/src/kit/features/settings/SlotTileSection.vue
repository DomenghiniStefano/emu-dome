<script setup lang="ts">
// The quick-save slot (V02): thumbnail, name, date, «Gioca da qui» and «…», four to a row as the
// Saves panel lays them out; and its thumbnail alone (SaveThumb), letterboxed or cropped.
import SaveThumb from '@/components/saves/SaveThumb.vue'
import SlotTile from '@/components/saves/SlotTile.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import SettingsSlotTileProposal from '@/kit/proposals/SettingsSlotTileProposal.vue'
import { THUMB_GBA, THUMB_NDS, THUMB_TALL } from './mock'
</script>

<template>
  <KitSection title="Quick-save slot" :group="KitGroup.Settings">
    <KitState
      class="w-full"
      label="Four to a row (Saves panel): auto, saved, outdated, no thumbnail"
    >
      <div class="grid w-full grid-cols-4 gap-8">
        <SlotTile name="Automatico" when="Oggi, 18:32" :thumb-url="THUMB_GBA" />
        <SlotTile name="Slot 1" when="Ieri, 22:10" :thumb-url="THUMB_NDS" />
        <SlotTile name="Slot 2" when="3 nov 2026, 09:05" :thumb-url="THUMB_GBA" outdated />
        <SlotTile name="Slot 3" when="1 nov 2026, 14:47" />
      </div>
    </KitState>
    <KitState class="w-full" label="Empty slot next to a saved one, portrait thumbnail letterboxed">
      <div class="grid w-full grid-cols-4 gap-8">
        <SlotTile name="Slot 4" when="Oggi, 18:40" :thumb-url="THUMB_TALL" />
        <SlotTile name="Slot 5" empty />
      </div>
    </KitState>
    <KitState class="w-full" label="SaveThumb · letterboxed (default), cropped, none">
      <div class="flex items-start gap-12">
        <SaveThumb :src="THUMB_TALL" class="aspect-4/3 w-cover-small rounded-control" />
        <SaveThumb :src="THUMB_TALL" crop class="aspect-4/3 w-cover-small rounded-control" />
        <SaveThumb :src="null" class="aspect-4/3 w-cover-small rounded-control" />
      </div>
    </KitState>

    <template #review>
      <KitIssue title="Salvataggio senza immagine: un riquadro nero" :status="IssueStatus.Proposed">
        Senza immagine SaveThumb mostra il nero del letterbox: «Slot 3» sembra un gioco fermo su una
        schermata nera, non un salvataggio senza anteprima. Lo stesso nel conflitto («Galaxy A71») e
        nelle miniature ritagliate.
        <template #current>
          <div class="w-tile-min">
            <SlotTile name="Slot 3" when="1 nov 2026, 14:47" />
          </div>
        </template>
        <template #proposed>
          <div class="w-tile-min">
            <SettingsSlotTileProposal name="Slot 3" when="1 nov 2026, 14:47" />
          </div>
        </template>
        <template #solution>
          in SaveThumb.vue, senza `src`: il fondo `bg-surface-raised` e l’icona ImageOff al centro
          in `text-ink-faint`, al posto di `bg-letterbox`. Il nero resta solo intorno a un’immagine
          vera.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
