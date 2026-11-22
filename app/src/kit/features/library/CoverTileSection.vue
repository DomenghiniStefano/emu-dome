<script setup lang="ts">
// CoverTile (library/CoverTile.vue): one game of the library's grid, in a cell as narrow as the
// grid lets it be (capsule-medium). «Gioca» and «…» show on hover or focus: move onto a tile with
// the arrows to see them.
import CoverTile from '@/components/library/CoverTile.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { KitGroup } from '@/kit/review'
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
      <KitIssue title="L’aspetto «non raggiungibile» è riscritto a mano">
        CoverTile non passa <code>missing</code> a GameCover: ripete nel suo <code>:class</code> la
        stessa stringa (<code>opacity-missing outline … outline-dashed</code>) che GameCover applica
        già con <code>missing</code>. Oggi i due aspetti coincidono, ma una modifica a uno solo dei
        due (per esempio la proposta sul tema Chiaro in «Game cover») li separa senza che nessun
        controllo se ne accorga.
        <template #solution>
          in <code>library/CoverTile.vue</code> sostituire il <code>:class</code> della copertina
          con <code>:missing="off"</code>. Nessuna differenza visibile.
        </template>
      </KitIssue>
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
    </template>
  </KitSection>
</template>
