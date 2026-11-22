<script setup lang="ts">
// ContinueCard (library/ContinueCard.vue): one game of «Continua a giocare», at the narrowest
// width of its grid (tile-min) and wider.
import ContinueCard from '@/components/library/ContinueCard.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import LibraryContinueCardProposal from '@/kit/proposals/LibraryContinueCardProposal.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
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
        title="La miniatura non è 3:2: si allunga con la scheda"
        :status="IssueStatus.Proposed"
      >
        La miniatura dovrebbe essere 3:2 (96 × 64), ma nella riga della scheda si stira fino
        all’altezza della scheda: diventa un rettangolo verticale tagliato ai lati, e la sua forma
        cambia da una scheda all’altra (più alta quando «Riparte da» va su due righe o i pulsanti
        vanno a capo). Intanto il titolo è troncato dopo poche lettere («Pocket Quest Advan…») anche
        se la scheda ha spazio in altezza.
        <template #current>
          <div class="w-tile-min">
            <ContinueCard :game="GBA" :point="RESUME_IN_GAME" :main="false" />
          </div>
        </template>
        <template #proposed>
          <div class="w-tile-min">
            <LibraryContinueCardProposal :game="GBA" :point="RESUME_IN_GAME" :main="false" />
          </div>
        </template>
        <template #solution>
          in <code>library/ContinueCard.vue</code> aggiungere <code>self-start</code> alla SaveThumb
          (resta 3:2 in alto) e sul titolo <code>line-clamp-2 wrap-anywhere</code> al posto di
          <code>truncate</code>. Sotto la miniatura resta uno spazio libero: se non va bene,
          l’alternativa è la miniatura alta quanto la scheda ma con la sua immagine intera (senza
          <code>crop</code>).
        </template>
      </KitIssue>
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
    </template>
  </KitSection>
</template>
