<script setup lang="ts">
// OverrideRow (details/OverrideRow.vue): L06, one setting with a value of this game's own, in the
// card DetailsSettings puts the rows in.
import DetailsCard from '@/components/details/DetailsCard.vue'
import OverrideRow from '@/components/details/OverrideRow.vue'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import LibraryOverrideRowProposal from '@/design-system/kit/proposals/LibraryOverrideRowProposal.vue'
import { IssueStatus, KitGroup } from '@/design-system/kit/review'
import { OVERRIDES } from './mock'

const noop = () => {}
</script>

<template>
  <KitSection title="Override row" :group="KitGroup.Library">
    <KitState label="Desktop: a console value, none, a long name and value">
      <div class="w-page-md max-w-full">
        <DetailsCard class="divide-y divide-line overflow-hidden p-0">
          <OverrideRow
            v-for="r in OVERRIDES"
            :key="r.key"
            :row="r"
            console-name="Game Boy Advance"
            :phone="false"
            @reset="noop"
          />
        </DetailsCard>
      </div>
    </KitState>
    <KitState label="Phone structure (desktop density): the value under the name">
      <div class="w-tile-min">
        <DetailsCard class="divide-y divide-line overflow-hidden p-0">
          <OverrideRow
            v-for="r in OVERRIDES"
            :key="r.key"
            :row="r"
            console-name="Game Boy Advance"
            phone
            @reset="noop"
          />
        </DetailsCard>
      </div>
    </KitState>
    <template #review>
      <KitIssue title="Ogni valore della lista è in ambra" :status="IssueStatus.Proposed">
        Il valore del gioco è scritto in <code>accent-text</code> semibold su ogni riga: con tre o
        quattro impostazioni la scheda diventa una colonna di testi ambra, mentre la regola del
        design system vuole l’ambra solo sull’azione principale. Il pallino a sinistra dice già che
        la riga è modificata.
        <template #current>
          <div class="w-dialog max-w-full">
            <DetailsCard class="divide-y divide-line overflow-hidden p-0">
              <OverrideRow
                v-for="r in OVERRIDES.slice(0, 2)"
                :key="r.key"
                :row="r"
                console-name="Game Boy Advance"
                :phone="false"
                @reset="noop"
              />
            </DetailsCard>
          </div>
        </template>
        <template #proposed>
          <div class="w-dialog max-w-full">
            <DetailsCard class="divide-y divide-line overflow-hidden p-0">
              <LibraryOverrideRowProposal
                v-for="r in OVERRIDES.slice(0, 2)"
                :key="r.key"
                :row="r"
                console-name="Game Boy Advance"
              />
            </DetailsCard>
          </div>
        </template>
        <template #solution>
          in <code>details/OverrideRow.vue</code> il valore (desktop e telefono) in
          <code>text-ink</code> semibold; l’ambra resta solo sul pallino, come nelle righe delle
          Impostazioni (SettingRowText).
        </template>
      </KitIssue>
      <KitIssue title="Il pallino «modificato» è disegnato a mano in tre posti">
        Lo stesso segno (<code>size-6 rounded-full bg-accent-text</code>) è scritto in
        <code>details/OverrideRow.vue</code>, <code>settings/SettingRowText.vue</code> e, a 8 px, in
        <code>settings/tv/TvSettingRow.vue</code>. Cambiarne colore o misura vuol dire trovarli
        tutti e tre.
        <template #solution>
          un componente unico (per esempio <code>settings/ChangedDot.vue</code>, importato da
          details) con una misura TV, usato dalle tre righe. Nessuna differenza visibile.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
