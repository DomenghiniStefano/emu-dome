<script setup lang="ts">
// CoverTile (library/CoverTile.vue): one game of the library's grid, in a cell as narrow as the
// grid lets it be (capsule-medium). «Gioca» and «…» show on hover or focus: move onto a tile with
// the arrows to see them.
import CoverTile from '@/components/library/CoverTile.vue'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'
import { GB_LONG, GBA, GBC_MISSING, N3DS_USER, NDS_ICON } from './mock'
</script>

<template>
  <KitSection title="Cover tile" :group="KitGroup.Library">
    <KitState label="Played (GBA)">
      <div class="w-capsule-medium"><CoverTile :game="GBA" :menu-open="false" /></div>
    </KitState>
    <KitState label="ROM icon (DS)">
      <div class="w-capsule-medium"><CoverTile :game="NDS_ICON" :menu-open="false" /></div>
    </KitState>
    <KitState label="User image, 3 regional versions">
      <div class="w-capsule-medium">
        <CoverTile :game="N3DS_USER" :menu-open="false" :versions="3" />
      </div>
    </KitState>
    <KitState label="Long title, never played">
      <div class="w-capsule-medium"><CoverTile :game="GB_LONG" :menu-open="false" /></div>
    </KitState>
    <KitState label="Folder not reachable (E01)">
      <div class="w-capsule-medium"><CoverTile :game="GBC_MISSING" :menu-open="false" /></div>
    </KitState>
    <template #review>
      <KitIssue title="Il titolo compare due volte con la copertina generata">
        Con la copertina generata (o l’icona della ROM) il titolo è scritto sulla copertina e subito
        sotto, nella stessa riga di testo: nella griglia ogni gioco senza immagine ripete il proprio
        nome, e con un titolo lungo i due blocchi occupano metà dell’altezza della cella. Con
        l’immagine dell’utente invece compare una volta sola.
        <template #solution>
          da decidere col proprietario: o la copertina generata nella griglia non scrive il titolo
          (<code>:title="false"</code>, resta il colore con il badge), o sotto la copertina generata
          resta solo la riga con tempo e ultima partita. Il comportamento va poi ripreso dagli
          scaffali TV.
        </template>
      </KitIssue>
      <KitIssue title="Con pochi giochi la griglia occupa solo la parte sinistra">
        La griglia tiene ogni copertina fra 156 px e la larghezza che le resta, e riserva le colonne
        anche quando sono vuote (<code>auto-fill</code>): a 1920 × 1080 con 5 giochi le copertine
        prendono metà della riga e a destra resta una fascia vuota fino alla barra «Ordina per» /
        «Aggiungi giochi». Con <code>auto-fit</code> le cinque copertine si allargherebbero a tutta
        la riga, ma a circa 350 × 525 px ciascuna (una sola riga visibile).
        <template #solution>
          da decidere col proprietario: lasciare la griglia com’è (la fascia sparisce appena i
          giochi riempiono una riga), oppure dare alla copertina anche una larghezza massima (un
          token nuovo) e allargarle fino a quella.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
