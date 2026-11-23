<script setup lang="ts">
// C03's shortcut list: the action, what to press (with its badge when reserved) and the row's
// «Cambia»; a reserved row and a keyboard-only one in muted ink.
import { ref } from 'vue'
import { Card, CardVariant } from '@/design-system/components/card'
import { CaptureField, CaptureFieldVariant } from '@/design-system/components/capture-field'
import ShortcutRow from '@/components/controls/ShortcutRow.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'

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
  </KitSection>
</template>
