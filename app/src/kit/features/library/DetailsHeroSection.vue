<script setup lang="ts">
// DetailsHero (details/DetailsHero.vue): the head of the game details (L04) on the desktop, with
// ResumeButton (its main action) on its own too. The phone's head is laid out by `touch:` classes
// that follow `data-density` on <html>, which the kit does not switch.
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import DetailsHero from '@/components/details/DetailsHero.vue'
import ResumeButton from '@/components/details/ResumeButton.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import { GB_LONG, GBA, GBC_MISSING, N3DS_USER, RESUME_AUTO, RESUME_NONE } from './mock'

const FAVOURITE = { ...N3DS_USER, favourite: true }
</script>

<template>
  <KitSection title="Details hero" :group="KitGroup.Library">
    <KitState label="Played, a quick save to resume">
      <div class="w-page-md max-w-full">
        <DetailsHero :game="GBA" :phone="false" :resume="RESUME_AUTO" />
      </div>
    </KitState>
    <KitState label="Favourite, user cover (3DS)">
      <div class="w-page-md max-w-full">
        <DetailsHero :game="FAVOURITE" :phone="false" :resume="RESUME_AUTO" />
      </div>
    </KitState>
    <KitState label="Never played, long title: «Gioca»">
      <div class="w-page-md max-w-full">
        <DetailsHero :game="GB_LONG" :phone="false" :resume="RESUME_NONE" />
      </div>
    </KitState>
    <KitState label="File missing: faded «Riprendi», the reason under it">
      <div class="w-page-md max-w-full">
        <DetailsHero :game="GBC_MISSING" :phone="false" :resume="RESUME_AUTO" />
      </div>
    </KitState>
    <KitState label="Resume button · Riprendi / Gioca / file missing">
      <div class="flex gap-8">
        <ResumeButton available :fresh="false" />
        <ResumeButton available fresh />
        <ResumeButton :available="false" :fresh="false" />
      </div>
    </KitState>
    <template #review>
      <KitIssue
        title="File mancante: la copertina grande non lo mostra"
        :status="IssueStatus.Proposed"
      >
        Nella griglia un gioco col file non raggiungibile ha la copertina sbiadita e tratteggiata;
        aprendo i suoi dettagli la copertina grande torna piena, uguale a quella di un gioco
        avviabile. Lo dicono solo «Riprendi» sbiadito e la riga arancione piccola sotto i pulsanti.
        <template #current>
          <GameCover :game="GBC_MISSING" :size="CoverSize.Hero" :badge="false" />
        </template>
        <template #proposed>
          <GameCover :game="GBC_MISSING" :size="CoverSize.Hero" :badge="false" missing />
        </template>
        <template #solution>
          in <code>details/DetailsHero.vue</code> passare <code>:missing="!game.available"</code>
          alla GameCover Hero: lo stesso aspetto della griglia (e della proposta sul tema Chiaro in
          «Game cover», se accettata).
        </template>
      </KitIssue>
      <KitIssue
        title="Il titolo è scritto due volte, uno accanto all’altro"
        :status="IssueStatus.Proposed"
      >
        Con la copertina generata il titolo compare sulla copertina e, 28 px più a destra, nel
        titolo grande della pagina. Con un titolo lungo i due blocchi di testo si affiancano uguali
        (vedi «Never played, long title» sopra).
        <template #current>
          <GameCover :game="GB_LONG" :size="CoverSize.Hero" :badge="false" />
        </template>
        <template #proposed>
          <GameCover :game="GB_LONG" :size="CoverSize.Hero" :badge="false" :title="false" />
        </template>
        <template #solution>
          in <code>details/DetailsHero.vue</code> <code>:title="false"</code> sulla copertina Hero:
          restano il colore del gioco e il bagliore intorno, il titolo lo dice già l’h1 accanto. Sul
          telefono (copertina 96 px accanto al titolo) vale lo stesso.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
