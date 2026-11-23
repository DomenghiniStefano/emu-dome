<script setup lang="ts">
// LinkRow: rows of a List card that lead one step down (Settings T01, Controls C01, the licences
// page), with a value, disabled, as a row that only says where a thing is done (`as="div"`),
// long text, and on the phone.
import { Info } from '@lucide/vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Card, CardVariant } from '@/design-system/components/card'
import { LinkRow } from '@/design-system/components/link-row'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus } from '@/design-system/kit/review'
</script>

<template>
  <KitSection title="LinkRow">
    <KitState label="Name and line; with a value; disabled; as a div (Controls C01)">
      <Card :variant="CardVariant.List" class="w-menu max-w-full">
        <LinkRow name="Remap the buttons" sub="For every game, per console or for one game" />
        <LinkRow name="Screen effect" sub="Screen effects" value="CRT TV" />
        <LinkRow name="Sticks, triggers and rumble" sub="Dead zone 12 % · triggers 30 %" disabled />
        <LinkRow as="div" name="Touch controls" sub="Customised from the phone" />
      </Card>
    </KitState>
    <KitState label="Licences page: many rows, the first one data-nav-initial">
      <Card :variant="CardVariant.List" class="w-menu max-w-full">
        <LinkRow name="EmuDome" sub="GPL-3.0-or-later · version 0.1.0" />
        <LinkRow name="mGBA" sub="Game Boy, Game Boy Color, Game Boy Advance · MPL-2.0" />
        <LinkRow name="melonDS DS" sub="Nintendo DS · GPL-3.0-or-later" />
      </Card>
    </KitState>
    <KitState label="Long name, line and value in a narrow card">
      <Card :variant="CardVariant.List" class="w-field">
        <LinkRow
          name="Velocità dell'avanzamento veloce"
          sub="Velocità · modificata 3 giorni fa"
          value="Massima, senza limite"
        />
      </Card>
    </KitState>
    <KitState label="On the phone (touch density)">
      <div data-density="touch" class="flex">
        <Card :variant="CardVariant.List" class="w-field">
          <LinkRow name="Remap the buttons" sub="For every game" />
          <LinkRow name="Screen effect" sub="Screen effects" value="CRT TV" />
        </Card>
      </div>
    </KitState>

    <template #review>
      <KitIssue
        title="La riga che non porta da nessuna parte ha la freccia"
        :status="IssueStatus.Proposed"
      >
        Con `as="div"` la riga dice soltanto dove si fa una cosa (Controlli › «Controlli touch: si
        personalizzano dal telefono»): non si preme e il controller non ci si ferma. Però disegna la
        stessa freccia › delle righe che aprono una pagina, quindi promette un passo che non c'è:
        col mouse si prova a cliccarla e non succede niente.
        <template #current>
          <Card :variant="CardVariant.List" class="w-field">
            <LinkRow name="Remap the buttons" sub="For every game" />
            <LinkRow as="div" name="Touch controls" sub="Customised from the phone" />
          </Card>
        </template>
        <template #proposed>
          <Card :variant="CardVariant.List" class="w-field">
            <LinkRow name="Remap the buttons" sub="For every game" />
            <Button
              as="div"
              :variant="ButtonVariant.LinkRow"
              :size="ButtonSize.LinkRow"
              :fade="ButtonFade.Dim"
            >
              <span class="flex min-w-0 flex-1 flex-col">
                <span class="font-medium text-ink">Touch controls</span>
                <span class="text-caption">Customised from the phone</span>
              </span>
              <Info :size="16" :stroke-width="1.75" aria-hidden="true" />
            </Button>
          </Card>
        </template>
        <template #solution>
          in design-system/components/link-row/LinkRow.vue la freccia solo quando la riga è un
          pulsante; con `as="div"` un'icona «Info» (o niente: da decidere) al suo posto.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
