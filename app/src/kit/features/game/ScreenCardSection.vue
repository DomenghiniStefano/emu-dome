<script setup lang="ts">
// G09's cards (Schermi e immagine): the setting's name, ‹ value ›, where the value comes from; a
// change «in prova» marked, a card that cannot change here saying why. In the bar the cards are
// a fifth of 1040 px each, so they are shown at that width.
import ScreenCard from '@/components/quick-menu/ScreenCard.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import GameScreenCardProposal from '@/kit/proposals/GameScreenCardProposal.vue'
import GameStage from './GameStage.vue'
import { SCREEN_CARDS } from './mock'
</script>

<template>
  <KitSection title="Screen card" :group="KitGroup.Game">
    <KitState
      label="As from Tutti i giochi · in prova (› at its end) · locked (no ‹ ›, the reason)"
    >
      <GameStage panel :scrim="false">
        <div class="grid grid-cols-3 gap-8">
          <ScreenCard :card="SCREEN_CARDS.plain" first />
          <ScreenCard :card="SCREEN_CARDS.trial" />
          <ScreenCard :card="SCREEN_CARDS.locked" />
        </div>
      </GameStage>
    </KitState>

    <template #review>
      <KitIssue title="«In prova» ha tre aspetti diversi nel menu" :status="IssueStatus.Proposed">
        La stessa cosa, una modifica in prova, si vede in tre modi: la card qui ha bordo accento e
        riempimento <code>violet-tint</code> (10% dell’accento, un token usato solo qui e nella riga
        di rimappatura in ascolto), il pannello delle modifiche (TrialPanel) ha bordo accento su
        superficie piena, e uno slot scelto ha bordo accento su <code>accent-soft</code> (18%). Così
        la card in prova sembra «scelta», non «da salvare».
        <template #current>
          <div class="w-card-min">
            <ScreenCard :card="SCREEN_CARDS.trial" />
          </div>
        </template>
        <template #proposed>
          <div class="w-card-min">
            <GameScreenCardProposal :card="SCREEN_CARDS.trial" />
          </div>
        </template>
        <template #solution>
          in ScreenCard.vue lo stato in prova diventa
          <code>border-accent-text bg-surface</code> (come TrialPanel) con la riga sotto in
          <code>font-semibold text-accent-text</code>. <code>violet-tint</code> resta solo alla riga
          in ascolto di RemapTableRow (vedi la sua sezione).
        </template>
      </KitIssue>
      <KitIssue
        title="Il motivo di una card bloccata viene tagliato"
        :status="IssueStatus.Proposed"
      >
        Una card che non si può cambiare qui scrive il motivo al posto della provenienza, ma la riga
        è <code>truncate</code>: nella barra (cinque card in 1040 px) di «Su questo sistema il
        metodo Accelerato non è disponibile.» si legge solo l’inizio, e il motivo è l’unica cosa che
        spiega perché ‹ › mancano.
        <template #current>
          <div class="w-card-min">
            <ScreenCard :card="SCREEN_CARDS.locked" />
          </div>
        </template>
        <template #proposed>
          <div class="w-card-min">
            <GameScreenCardProposal :card="SCREEN_CARDS.locked" />
          </div>
        </template>
        <template #solution>
          in ScreenCard.vue la riga sotto passa da <code>truncate</code> a
          <code>line-clamp-2</code>; le card di una riga della griglia restano alte uguali, perché
          la griglia le allunga già tutte alla più alta.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
