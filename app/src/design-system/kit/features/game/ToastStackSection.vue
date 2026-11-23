<script setup lang="ts">
// The short messages (G13, G17): bottom left in the shell, at the top centre in game. Each kind:
// a notice, a notice with a quieter note, one with an action, an error.
import { CircleAlert } from '@lucide/vue'
import { ToastKind } from '@/components/toasts/toasts'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/design-system/kit/review'
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
    <KitState label="Shell, bottom left past the rail: a notice, one with an action, an error">
      <div
        class="relative h-cover-hero w-panel max-w-full overflow-hidden rounded-card border border-line bg-bg transform-gpu"
      >
        <!-- The shell's rail: the stack starts past it. -->
        <div class="absolute inset-y-0 left-0 w-rail border-r border-line bg-glass-thin"></div>
        <ToastDemo :toasts="SHELL" />
      </div>
    </KitState>
    <KitState label="In game, top centre: a notice, with a note, with an action">
      <GameStage :scrim="false">
        <div class="h-cover-hero w-panel max-w-full">
          <ToastDemo :toasts="IN_GAME" in-game />
        </div>
      </GameStage>
    </KitState>
    <KitState label="A long text with an action">
      <div
        class="relative h-cover-small w-panel max-w-full overflow-hidden rounded-card border border-line bg-bg transform-gpu"
      >
        <!-- The shell's rail: the stack starts past it. -->
        <div class="absolute inset-y-0 left-0 w-rail border-r border-line bg-glass-thin"></div>
        <ToastDemo :toasts="LONG" />
      </div>
    </KitState>

    <template #review>
      <KitIssue title="Un errore si riconosce solo dal colore" :status="IssueStatus.Proposed">
        Il toast di errore è uguale a quello normale con il testo tutto in rosso: chi non distingue
        bene i colori non lo riconosce, e un testo lungo tutto rosso su vetro chiaro si legge male.
        <template #current>
          <div
            class="flex max-w-toast min-h-control-lg items-center gap-8 rounded-card py-6 pr-10 pl-10 font-medium glass-thick text-danger"
          >
            Impossibile leggere la cartella dei giochi. Controlla che il disco sia collegato.
          </div>
        </template>
        <template #proposed>
          <div
            class="flex max-w-toast min-h-control-lg items-center gap-8 rounded-card py-6 pr-10 pl-10 font-medium glass-thick"
          >
            <CircleAlert
              :size="18"
              :stroke-width="1.75"
              class="shrink-0 text-danger"
              aria-hidden="true"
            />
            <span class="min-w-0 flex-1"
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
    </template>
  </KitSection>
</template>
