<script setup lang="ts">
// C03's shortcut list: the action, what to press (with its badge when reserved) and the row's
// «Cambia»; a reserved row and a keyboard-only one in muted ink.
import { ref } from 'vue'
import { Card, CardVariant } from '@/components/ui/card'
import { CaptureField, CaptureFieldVariant } from '@/components/ui/capture-field'
import ShortcutRow from '@/components/controls/ShortcutRow.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import KitIssue from '@/kit/KitIssue.vue'
import { IssueStatus, KitGroup } from '@/kit/review'

const listening = ref<string | null>(null)
const toggle = (id: string) => (listening.value = listening.value === id ? null : id)
</script>

<template>
  <KitSection title="Shortcut row" :group="KitGroup.Game">
    <KitState label="Active (click Cambia: listening) · reserved with badge · keyboard only">
      <Card :variant="CardVariant.List" class="w-page max-w-full">
        <ShortcutRow action="Salvataggio rapido">
          L3 + R1
          <template #change>
            <CaptureField
              :variant="CaptureFieldVariant.Quiet"
              :listening="listening === 'save'"
              @click="toggle('save')"
            >
              Cambia
            </CaptureField>
          </template>
        </ShortcutRow>
        <ShortcutRow action="Avanti veloce finché tieni premuto">
          L3 + R2
          <template #change>
            <CaptureField
              :variant="CaptureFieldVariant.Quiet"
              :listening="listening === 'ff'"
              @click="toggle('ff')"
            >
              Cambia
            </CaptureField>
          </template>
        </ShortcutRow>
        <ShortcutRow action="Chiudi il coperchio" muted badge="Con Nintendo DS">
          L3 + Select
        </ShortcutRow>
        <ShortcutRow action="Riavvolgi" muted badge="In arrivo">L3 + L2</ShortcutRow>
        <ShortcutRow action="A schermo intero" muted>Tastiera: F11</ShortcutRow>
      </Card>
    </KitState>

    <template #review>
      <KitIssue title="Le righe della lista hanno altezze diverse" :status="IssueStatus.Proposed">
        Una riga con «Cambia» è alta quanto il campo (circa 48 px), una riga riservata o solo da
        tastiera, senza campo, è più bassa (circa 40 px). Nella stessa lista il ritmo salta proprio
        dove le righe riservate si alternano a quelle attive.
        <template #current>
          <Card :variant="CardVariant.List" class="w-dialog max-w-full">
            <ShortcutRow action="Salvataggio rapido">
              L3 + R1
              <template #change>
                <CaptureField :variant="CaptureFieldVariant.Quiet">Cambia</CaptureField>
              </template>
            </ShortcutRow>
            <ShortcutRow action="Riavvolgi" muted badge="In arrivo">L3 + L2</ShortcutRow>
            <ShortcutRow action="A schermo intero" muted>Tastiera: F11</ShortcutRow>
          </Card>
        </template>
        <template #proposed>
          <Card :variant="CardVariant.List" class="w-dialog max-w-full">
            <ShortcutRow action="Salvataggio rapido" class="min-h-48">
              L3 + R1
              <template #change>
                <CaptureField :variant="CaptureFieldVariant.Quiet">Cambia</CaptureField>
              </template>
            </ShortcutRow>
            <ShortcutRow action="Riavvolgi" muted badge="In arrivo" class="min-h-48"
              >L3 + L2</ShortcutRow
            >
            <ShortcutRow action="A schermo intero" muted class="min-h-48"
              >Tastiera: F11</ShortcutRow
            >
          </Card>
        </template>
        <template #solution>
          in ShortcutRow.vue un’altezza minima sulla riga (<code>min-h-48</code>, o un token di
          ruolo <code>--spacing-list-row</code>), così ogni riga è alta uguale con o senza campo.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
