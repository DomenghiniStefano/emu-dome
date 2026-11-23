<script setup lang="ts">
// GameCover (cover/GameCover.vue): what each cover kind shows, at the sizes the screens use.
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import LibraryGameCoverProposal from '@/design-system/kit/proposals/LibraryGameCoverProposal.vue'
import { IssueStatus, KitGroup } from '@/design-system/kit/review'
import { GB_LONG, GBA, GBC_MISSING, N3DS_USER, NDS_ICON } from './mock'

const BROKEN = { ...N3DS_USER, coverUrl: 'emu://cover/broken.jpg' }
</script>

<template>
  <KitSection title="Game cover" :group="KitGroup.Library">
    <KitState label="Fill · generated (GBA)">
      <div class="w-capsule-medium"><GameCover :game="GBA" /></div>
    </KitState>
    <KitState label="Fill · ROM icon (DS)">
      <div class="w-capsule-medium"><GameCover :game="NDS_ICON" /></div>
    </KitState>
    <KitState label="Fill · the user's image (3DS)">
      <div class="w-capsule-medium"><GameCover :game="N3DS_USER" /></div>
    </KitState>
    <KitState label="Fill · user image that fails: generated">
      <div class="w-capsule-medium"><GameCover :game="BROKEN" /></div>
    </KitState>
    <KitState label="Fill · long title (GB)">
      <div class="w-capsule-medium"><GameCover :game="GB_LONG" /></div>
    </KitState>
    <KitState label="Fill · missing (only on Drive)">
      <div class="w-capsule-medium"><GameCover :game="GBC_MISSING" missing /></div>
    </KitState>
    <KitState label="Fill · 3 regional versions">
      <div class="w-capsule-medium"><GameCover :game="GBA" :versions="3" /></div>
    </KitState>
    <KitState label="Hero · Small · Row · Thumb">
      <div class="flex items-end gap-12">
        <GameCover :game="NDS_ICON" :size="CoverSize.Hero" :badge="false" />
        <GameCover :game="GB_LONG" :size="CoverSize.Small" />
        <GameCover :game="GBA" :size="CoverSize.Row" :badge="false" :title="false" />
        <GameCover :game="NDS_ICON" :size="CoverSize.Thumb" :badge="false" :title="false" />
      </div>
    </KitState>
    <template #review>
      <KitIssue
        title="Copertina «mancante» illeggibile nel tema Chiaro"
        :status="IssueStatus.Proposed"
      >
        <code>missing</code> sbiadisce tutta la copertina al 45%: sul fondo scuro resta leggibile,
        sul fondo chiaro diventa un rosa slavato con il titolo bianco quasi invisibile (provalo con
        il selettore Chiaro in alto). Lo stesso aspetto lo usano la griglia (cartella non
        raggiungibile) e lo scaffale «Solo su Google Drive».
        <template #current>
          <div class="w-capsule-medium"><GameCover :game="GBC_MISSING" missing /></div>
        </template>
        <template #proposed>
          <div class="w-capsule-medium">
            <LibraryGameCoverProposal :game="GBC_MISSING" missing />
          </div>
        </template>
        <template #solution>
          in <code>GameCover.vue</code> il ramo <code>missing</code> usa <code>grayscale</code> al
          posto di <code>opacity-missing</code>, con il bordo tratteggiato di adesso: il colore
          sparisce, il contrasto del titolo resta in tutti e tre i temi. Il testo sotto la copertina
          (CoverTile) resta sbiadito come ora.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
