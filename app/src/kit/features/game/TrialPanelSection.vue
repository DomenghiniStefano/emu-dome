<script setup lang="ts">
// The changes «in prova» (G16, G10, the phone's G09): what they are, that closing the game drops
// them, and the three answers. On the desktop's G09 bar the same answers are one row (TrialActions
// beside the title, ScreensBar.vue), shown last for comparison.
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import TrialActions from '@/components/quick-menu/TrialActions.vue'
import TrialPanel from '@/components/quick-menu/TrialPanel.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import GameStage from './GameStage.vue'
</script>

<template>
  <KitSection title="Trial panel" :group="KitGroup.Game">
    <KitState label="One change">
      <GameStage panel>
        <TrialPanel title="1 modifica in prova: Morbido" console-name="Game Boy Advance" />
      </GameStage>
    </KitState>
    <KitState label="Three changes, long names">
      <GameStage panel>
        <TrialPanel
          title="3 modifiche in prova: Riempi lo schermo, Morbido, CRT curvo"
          console-name="Nintendo DS"
        />
      </GameStage>
    </KitState>
    <KitState label="G09 bar on the desktop: the title and two answers in a row">
      <GameStage panel :scrim="false">
        <div class="flex items-center gap-8">
          <span class="min-w-0 flex-1 font-semibold">1 modifica in prova: Morbido</span>
          <TrialActions />
        </div>
      </GameStage>
    </KitState>

    <template #review>
      <KitIssue
        title="Il terzo pulsante è un link restilizzato a mano"
        :status="IssueStatus.Proposed"
      >
        «Salva invece per tutti i giochi …» è un Button <code>Link</code> con in più la classe
        <code>underline</code> passata dal componente: è l’unico link sottolineato fisso dell’app
        (gli altri si sottolineano al passaggio) e va contro la regola «uno schermo non restilizza
        una primitiva».
        <template #current>
          <Button :variant="ButtonVariant.Link" :size="ButtonSize.Text" class="underline">
            Salva invece per tutti i giochi Game Boy Advance
          </Button>
        </template>
        <template #proposed>
          <Button :variant="ButtonVariant.Link" :size="ButtonSize.Text">
            Salva invece per tutti i giochi Game Boy Advance
          </Button>
        </template>
        <template #solution>
          togliere <code>underline</code> in TrialPanel.vue e usare la variante com’è. Se il
          proprietario vuole il link sempre sottolineato, diventa una variante nuova in
          <code>button/variants.ts</code>, provata prima qui nel Kit.
        </template>
      </KitIssue>
      <KitIssue
        title="Sulla barra di G09 manca «Salva per tutti i giochi»"
        :status="IssueStatus.Open"
      >
        Il pannello offre tre risposte (annulla, salva per il gioco, salva per la console); la barra
        di G09 sul desktop (ultimo stato) ne offre due, e per salvare un filtro per tutta la console
        bisogna passare dalle Impostazioni. Da verificare se è voluto (spazio nella barra) o una
        dimenticanza: le stesse modifiche hanno risposte diverse a seconda della pagina.
        <template #solution>
          da decidere: aggiungere il link della console anche in ScreensBar.vue (dopo i due
          pulsanti, come in TrialPanel), oppure scrivere nel commento di ScreensBar perché manca.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
