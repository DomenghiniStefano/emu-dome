<script setup lang="ts">
// C02's remap table: the game button, the device's button that gives it (a capture field that
// listens when pressed) and the status (used twice, changed); under a row that asked, the
// conflict banner. The head row is written as ControlsRemap.vue writes it.
import { ref } from 'vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import ConflictBanner from '@/components/controls/ConflictBanner.vue'
import RemapTableRow from '@/components/controls/RemapTableRow.vue'
import { RemapRowStatus } from '@/components/controls/remapTable'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import GameRemapTableRowProposal from '@/kit/proposals/GameRemapTableRowProposal.vue'

const conflict = ref(false)
const ROWS = [
  { button: 'A', source: 'B (destra)', status: RemapRowStatus.None },
  { button: 'B', source: 'A (sotto)', status: RemapRowStatus.Changed },
  { button: 'L', source: 'LB', status: RemapRowStatus.UsedTwice },
  { button: 'R', source: 'RB', status: RemapRowStatus.None },
]
const STATUS_TEXT: Record<RemapRowStatus, string> = {
  [RemapRowStatus.None]: '',
  [RemapRowStatus.Changed]: 'Modificato',
  [RemapRowStatus.UsedTwice]: 'Usato anche da un altro',
}
</script>

<template>
  <KitSection title="Remap table row" :group="KitGroup.Game">
    <KitState label="Plain · changed · used twice · plain, and a row listening (keyboard, 4 s)">
      <Card :variant="CardVariant.List" class="w-page max-w-full">
        <div
          class="grid grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)_auto] items-center gap-12 bg-surface-raised px-16 py-8 text-overline text-ink-muted"
        >
          <span>NEL GIOCO</span>
          <span>SUL CONTROLLER</span>
          <span></span>
        </div>
        <RemapTableRow
          v-for="r in ROWS"
          :key="r.button"
          v-bind="r"
          :listening="false"
          :seconds-left="0"
          :keyboard="false"
        />
        <RemapTableRow
          button="Start"
          source="Invio"
          :status="RemapRowStatus.None"
          listening
          :seconds-left="4"
          keyboard
        />
      </Card>
    </KitState>
    <KitState label="The conflict banner under the row (it takes the focus when shown)">
      <div class="flex w-page max-w-full flex-col gap-8">
        <Button class="self-start" @click="conflict = !conflict">
          {{ conflict ? 'Hide the banner' : 'Show the banner' }}
        </Button>
        <Card v-if="conflict" :variant="CardVariant.List">
          <RemapTableRow
            button="L"
            source="LB"
            :status="RemapRowStatus.None"
            :listening="false"
            :seconds-left="0"
            :keyboard="false"
          />
          <ConflictBanner
            text="LB è già usato per «R». Vuoi scambiarli?"
            @swap="conflict = false"
            @cancel="conflict = false"
          />
        </Card>
      </div>
    </KitState>

    <template #review>
      <KitIssue
        title="Le colonne della tabella non sono allineate fra le righe"
        :status="IssueStatus.Proposed"
      >
        Ogni riga è una griglia a sé con la terza colonna <code>auto</code>: la riga «Usato anche da
        un altro» ha una terza colonna larga, «Modificato» media, le altre zero. Le prime due
        colonne si dividono quello che resta, quindi i campi del controller partono in un punto
        diverso in ogni riga e non sono in colonna con l’intestazione. ShortcutRow risolve lo stesso
        caso con una colonna fissa (<code>--spacing-shortcut-col</code>).
        <template #current>
          <Card :variant="CardVariant.List" class="w-dialog max-w-full">
            <div
              class="grid grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)_auto] items-center gap-12 bg-surface-raised px-16 py-8 text-overline text-ink-muted"
            >
              <span>NEL GIOCO</span>
              <span>SUL CONTROLLER</span>
              <span></span>
            </div>
            <RemapTableRow
              v-for="r in ROWS"
              :key="r.button"
              v-bind="r"
              :listening="false"
              :seconds-left="0"
              :keyboard="false"
            />
          </Card>
        </template>
        <template #proposed>
          <Card :variant="CardVariant.List" class="w-dialog max-w-full">
            <GameRemapTableRowProposal head />
            <GameRemapTableRowProposal
              v-for="r in ROWS"
              :key="r.button"
              :button="r.button"
              :source="r.source"
              :status="STATUS_TEXT[r.status]"
              :warn="r.status === RemapRowStatus.UsedTwice"
            />
          </Card>
        </template>
        <template #solution>
          in RemapTableRow.vue e nella riga d’intestazione di ControlsRemap.vue la terza colonna da
          <code>auto</code> a una larghezza fissa: un token di ruolo nuovo
          <code>--spacing-remap-status</code> (150, qui mostrato con
          <code>--spacing-shortcut-key</code> che vale uguale). La riga in ascolto resta com’è.
        </template>
      </KitIssue>
      <KitIssue title="«Scambia» e «Annulla» sono uguali" :status="IssueStatus.Proposed">
        Nel banner del conflitto i due pulsanti hanno lo stesso aspetto: la risposta attesa (il
        focus parte su «Scambia») non si distingue da quella che lascia tutto com’era.
        <template #current>
          <div
            class="flex w-dialog max-w-full items-center gap-8 border-t border-warning-edge bg-warning-tint px-16 py-8"
          >
            <span class="flex-1">LB è già usato per «R». Vuoi scambiarli?</span>
            <Button>Scambia</Button>
            <Button>Annulla</Button>
          </div>
        </template>
        <template #proposed>
          <div
            class="flex w-dialog max-w-full items-center gap-8 border-t border-warning-edge bg-warning-tint px-16 py-8"
          >
            <span class="flex-1">LB è già usato per «R». Vuoi scambiarli?</span>
            <Button :variant="ButtonVariant.Strong">Scambia</Button>
            <Button :variant="ButtonVariant.Ghost">Annulla</Button>
          </div>
        </template>
        <template #solution>
          in ConflictBanner.vue «Scambia» con <code>ButtonVariant.Strong</code> e «Annulla» con
          <code>ButtonVariant.Ghost</code> (l’accento resta alle azioni principali della pagina).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
