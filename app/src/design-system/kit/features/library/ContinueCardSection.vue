<script setup lang="ts">
// ContinueCard (library/ContinueCard.vue): one game of «Continua a giocare», at the narrowest
// width of its grid (tile-min) and wider.
import ContinueCard from '@/components/library/ContinueCard.vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/design-system/kit/review'
import {
  GB_LONG,
  GBA,
  GBC_MISSING,
  NDS_ICON,
  RESUME_AUTO,
  RESUME_IN_GAME,
  RESUME_NONE,
  RESUME_SLOT,
} from './mock'

const LONG_PLAYED = { ...GB_LONG, lastPlayed: Date.now() - 3_600_000, playTimeSecs: 600 }
</script>

<template>
  <KitSection title="Continue card" :group="KitGroup.Library">
    <KitState label="First card: main action, from the auto save">
      <div class="w-tile-min"><ContinueCard :game="GBA" :point="RESUME_AUTO" main /></div>
    </KitState>
    <KitState label="Another card, from slot 3">
      <div class="w-tile-min">
        <ContinueCard :game="NDS_ICON" :point="RESUME_SLOT" :main="false" />
      </div>
    </KitState>
    <KitState label="In-game save only: no picture">
      <div class="w-tile-min">
        <ContinueCard :game="GBA" :point="RESUME_IN_GAME" :main="false" />
      </div>
    </KitState>
    <KitState label="Nothing saved: «Gioca», long title">
      <div class="w-tile-min">
        <ContinueCard :game="LONG_PLAYED" :point="RESUME_NONE" :main="false" />
      </div>
    </KitState>
    <KitState label="File not reachable">
      <div class="w-tile-min">
        <ContinueCard :game="GBC_MISSING" :point="RESUME_AUTO" :main="false" />
      </div>
    </KitState>
    <KitState label="Wide cell (dialog width)">
      <div class="w-dialog max-w-full">
        <ContinueCard :game="GBA" :point="RESUME_AUTO" :main="false" />
      </div>
    </KitState>
    <template #review>
      <KitIssue
        title="«Riprendi» spento senza spiegazione quando il file manca"
        :status="IssueStatus.Proposed"
      >
        Con il file non raggiungibile la scheda usa <code>disabled</code>: il pulsante non si può
        premere né raggiungere col controller, e nessuno dice perché. Gli altri due «Riprendi» /
        «Gioca» della libreria (dettagli e copertina della griglia) per lo stesso caso usano
        <code>aria-disabled</code>: restano sbiaditi ma premendoli spiegano che il file non si
        trova.
        <template #current>
          <Button :variant="ButtonVariant.Soft" :size="ButtonSize.Compact" disabled>
            Riprendi
          </Button>
        </template>
        <template #proposed>
          <Button :variant="ButtonVariant.Soft" :size="ButtonSize.Compact" aria-disabled="true">
            Riprendi
          </Button>
        </template>
        <template #solution>
          in <code>library/ContinueCard.vue</code> <code>:aria-disabled</code> al posto di
          <code>:disabled</code>. Il resto c’è già: <code>LibraryView</code> (<code>open</code>)
          alla pressione mostra <code>errors.libraryGameUnavailable</code>.
        </template>
      </KitIssue>
      <KitIssue title="Allargate a tutta la riga, le schede hanno spazio vuoto dentro">
        Per non lasciare una fascia vuota a destra le schede di «Continua a giocare» ora si
        allargano a tutta la riga (<code>auto-fit</code>, revisione del 2026-11-22). A 1920 px
        ciascuna è larga circa 600 px, ma immagine, titolo e pulsanti restano nei primi 250: il
        vuoto si è spostato dentro le schede. Con un solo gioco giocato la scheda è larga quanto la
        pagina. Le schede sono al massimo 4 (<code>CONTINUE_COUNT</code>): a 1024 px ne stanno 3 per
        riga e la quarta resta da sola sulla seconda, con due terzi di riga vuoti.
        <template #solution>
          da decidere col proprietario: l’immagine del salvataggio cresce con la scheda (3:2, fino a
          un massimo da decidere, un token nuovo), oppure le schede hanno una larghezza massima e il
          resto della riga mostra altri giochi recenti; e per la quarta scheda, mostrarne solo
          quante stanno in una riga.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
