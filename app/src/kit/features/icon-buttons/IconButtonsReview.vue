<script setup lang="ts">
// The review issues of Trello #104 under the Kit's «Button · icona» section: the rule in one
// paragraph, then one issue per area of the app with its text-only buttons now and as proposed.
import KitIssue from '@/kit/KitIssue.vue'
import { IssueStatus } from '@/kit/review'
import ReviewButtons from './ReviewButtons.vue'
import { REVIEW_GROUPS } from './review'
</script>

<template>
  <KitIssue title="La regola proposta (#104)" :status="IssueStatus.Proposed">
    Oggi 138 pulsanti su 186 sono solo testo. Proposta: l’azione principale di una vista e le azioni
    di finestre, schede, avvisi e piedi dei passi hanno un’icona Lucide davanti alle parole; le
    azioni ripetute su ogni riga o riquadro di una lista e le barre degli strumenti hanno l’icona
    sola, il testo di oggi come etichetta per l’accessibilità e suggerimento (nessun testo nuovo);
    link, scelte e schede restano testo. Totale: 100 icona e testo, 12 solo icona, 26 restano.
    <template #solution>
      il pulsante prende l’icona da sé (Button: icon, label per l’icona sola, la misura IconRow per
      le righe): le schermate non scrivono più dimensioni di icone. Sotto, un gruppo per area.
    </template>
  </KitIssue>
  <KitIssue
    v-for="group in REVIEW_GROUPS"
    :key="group.title"
    :title="group.title"
    :status="IssueStatus.Proposed"
  >
    {{ group.problem }}
    <template #current><ReviewButtons :rows="group.rows" side="now" /></template>
    <template #proposed><ReviewButtons :rows="group.rows" side="next" /></template>
    <template #solution>{{ group.solution }}</template>
  </KitIssue>
</template>
