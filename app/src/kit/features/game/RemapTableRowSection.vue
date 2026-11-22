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

const conflict = ref(false)
const ROWS = [
  { button: 'A', source: 'B (destra)', status: RemapRowStatus.None },
  { button: 'B', source: 'A (sotto)', status: RemapRowStatus.Changed },
  { button: 'L', source: 'LB', status: RemapRowStatus.UsedTwice },
  { button: 'R', source: 'RB', status: RemapRowStatus.None },
]
</script>

<template>
  <KitSection title="Remap table row" :group="KitGroup.Game">
    <KitState label="Plain · changed · used twice · plain, and a row listening (keyboard, 4 s)">
      <Card :variant="CardVariant.List" class="w-page max-w-full">
        <div
          class="grid grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)_var(--spacing-shortcut-key)] items-center gap-8 bg-surface-raised px-12 py-6 text-overline text-ink-muted"
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
