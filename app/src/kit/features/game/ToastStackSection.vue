<script setup lang="ts">
// The short messages (G13, G17): bottom left in the shell, at the top centre in game. Each kind:
// a notice, a notice with a quieter note, one with an action, an error.
import { CircleAlert } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { ToastKind } from '@/components/toasts/toasts'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import GameStage from './GameStage.vue'
import ToastDemo from './ToastDemo.vue'

const noop = () => {}
const SHELL = [
  { text: 'Screenshot salvato', kind: ToastKind.Info },
  { text: 'Copertina aggiornata', kind: ToastKind.Info, action: { label: 'Annulla', run: noop } },
  {
    text: 'Impossibile leggere la cartella dei giochi. Controlla che il disco sia collegato.',
    kind: ToastKind.Error,
  },
]
const IN_GAME = [
  { text: 'Salvato nello slot 2', kind: ToastKind.Info },
  { text: 'Caricato lo slot 2', kind: ToastKind.Info, note: 'Annulla dal menu rapido' },
  { text: 'Caricato lo slot 2', kind: ToastKind.Info, action: { label: 'Annulla', run: noop } },
]
const LONG = [
  {
    text: 'Copertina salvata per Pocket Quest Advance: Remastered Edition',
    kind: ToastKind.Info,
    action: { label: 'Annulla', run: noop },
  },
]
</script>

<template>
  <KitSection title="Toast stack" :group="KitGroup.Game">
    <KitState label="Shell, bottom left: a notice, one with an action, an error">
      <div
        class="relative h-menu w-panel max-w-full overflow-hidden rounded-card border border-line bg-bg transform-gpu"
      >
        <ToastDemo :toasts="SHELL" />
      </div>
    </KitState>
    <KitState label="In game, top centre: a notice, with a note, with an action">
      <GameStage :scrim="false">
        <div class="h-menu w-panel max-w-full">
          <ToastDemo :toasts="IN_GAME" in-game />
        </div>
      </GameStage>
    </KitState>
    <KitState label="A long text with an action">
      <div
        class="relative h-cover-small w-panel max-w-full overflow-hidden rounded-card border border-line bg-bg transform-gpu"
      >
        <ToastDemo :toasts="LONG" />
      </div>
    </KitState>

    <template #review>
      <KitIssue title="Un toast con azione non va a capo" :status="IssueStatus.Proposed">
        Il toast con un pulsante ha <code>whitespace-nowrap</code> dentro <code>max-w-toast</code>
        (420 px): un testo lungo (il titolo di un gioco) esce dal vetro o spinge il pulsante fuori
        (terzo stato). In italiano capita presto.
        <template #current>
          <div
            class="flex max-w-toast min-h-touch items-center gap-12 rounded-12 py-6 pr-6 pl-16 font-semibold whitespace-nowrap glass-thick"
          >
            <span>Copertina salvata per Pocket Quest Advance: Remastered Edition</span>
            <Button :variant="ButtonVariant.RaisedAccent" :size="ButtonSize.Toast">Annulla</Button>
          </div>
        </template>
        <template #proposed>
          <div
            class="flex max-w-toast min-h-touch items-center gap-12 rounded-12 py-6 pr-6 pl-16 font-semibold glass-thick"
          >
            <span class="min-w-0 flex-1">
              Copertina salvata per Pocket Quest Advance: Remastered Edition
            </span>
            <Button :variant="ButtonVariant.RaisedAccent" :size="ButtonSize.Toast">Annulla</Button>
          </div>
        </template>
        <template #solution>
          in ToastStack.vue togliere <code>whitespace-nowrap</code> dalla classe del toast con
          azione e dare al testo <code>min-w-0 flex-1</code>: il testo va a capo, il pulsante resta
          intero a destra.
        </template>
      </KitIssue>
      <KitIssue title="Due forme di toast nella stessa pila" :status="IssueStatus.Proposed">
        Un toast semplice ha raggio 10, padding 14 / 10 e peso medio; uno con azione ha raggio 12,
        padding diverso e semibold. Impilati (primi due stati) hanno angoli e pesi diversi, come se
        fossero due componenti. Il raggio 12 è segnato «pending Cowork» nel componente.
        <template #current>
          <div class="flex flex-col items-start gap-8">
            <div class="rounded-10 px-14 py-10 font-medium glass-thick">Screenshot salvato</div>
            <div
              class="flex min-h-touch items-center gap-12 rounded-12 py-6 pr-6 pl-16 font-semibold whitespace-nowrap glass-thick"
            >
              <span>Copertina aggiornata</span>
              <Button :variant="ButtonVariant.RaisedAccent" :size="ButtonSize.Toast"
                >Annulla</Button
              >
            </div>
          </div>
        </template>
        <template #proposed>
          <div class="flex flex-col items-start gap-8">
            <div class="flex min-h-touch items-center rounded-card px-14 font-medium glass-thick">
              Screenshot salvato
            </div>
            <div
              class="flex min-h-touch items-center gap-12 rounded-card py-6 pr-6 pl-14 font-medium glass-thick"
            >
              <span>Copertina aggiornata</span>
              <Button :variant="ButtonVariant.RaisedAccent" :size="ButtonSize.Toast"
                >Annulla</Button
              >
            </div>
          </div>
        </template>
        <template #solution>
          in ToastStack.vue una sola forma: <code>rounded-card</code> (10), altezza minima
          <code>min-h-touch</code>, padding sinistro 14 e peso medio per tutti; il toast con azione
          aggiunge solo il pulsante.
        </template>
      </KitIssue>
      <KitIssue title="Un errore si riconosce solo dal colore" :status="IssueStatus.Proposed">
        Il toast di errore è uguale a quello normale con il testo tutto in rosso: chi non distingue
        bene i colori non lo riconosce, e un testo lungo tutto rosso su vetro chiaro si legge male.
        <template #current>
          <div class="max-w-toast rounded-10 px-14 py-10 font-medium text-danger glass-thick">
            Impossibile leggere la cartella dei giochi. Controlla che il disco sia collegato.
          </div>
        </template>
        <template #proposed>
          <div
            class="flex max-w-toast items-start gap-8 rounded-10 px-14 py-10 font-medium glass-thick"
          >
            <CircleAlert
              :size="18"
              :stroke-width="1.75"
              class="shrink-0 text-danger"
              aria-hidden="true"
            />
            <span
              >Impossibile leggere la cartella dei giochi. Controlla che il disco sia
              collegato.</span
            >
          </div>
        </template>
        <template #solution>
          in ToastStack.vue, per <code>ToastKind.Error</code> un’icona Lucide
          <code>CircleAlert</code> in <code>text-danger</code> prima del testo, e il testo in
          <code>text-ink</code>.
        </template>
      </KitIssue>
      <KitIssue
        title="Nella shell i toast si allungano al più largo"
        :status="IssueStatus.Proposed"
      >
        In basso a sinistra la pila è una colonna senza allineamento: ogni toast si allarga quanto
        il più largo (primo stato), così «Screenshot salvato» ha una fascia vuota a destra e
        «Annulla» resta attaccato al testo invece di stare in fondo. In gioco la pila è centrata e
        il problema non c’è.
        <template #current>
          <div class="flex w-toast max-w-full flex-col gap-8">
            <div class="rounded-10 px-14 py-10 font-medium glass-thick">Screenshot salvato</div>
            <div class="rounded-10 px-14 py-10 font-medium text-danger glass-thick">
              Impossibile leggere la cartella dei giochi. Controlla che il disco sia collegato.
            </div>
          </div>
        </template>
        <template #proposed>
          <div class="flex w-toast max-w-full flex-col items-start gap-8">
            <div class="rounded-10 px-14 py-10 font-medium glass-thick">Screenshot salvato</div>
            <div class="rounded-10 px-14 py-10 font-medium text-danger glass-thick">
              Impossibile leggere la cartella dei giochi. Controlla che il disco sia collegato.
            </div>
          </div>
        </template>
        <template #solution>
          in ToastStack.vue aggiungere <code>items-start</code> alla classe della pila fuori dal
          gioco (<code>bottom-16 left-16 items-start</code>): ogni toast largo quanto il suo testo.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
