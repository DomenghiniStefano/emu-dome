<script setup lang="ts">
import { Button, ButtonSize } from '@/components/ui/button'
import { CaptureField, CaptureFieldVariant } from '@/components/ui/capture-field'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'
</script>

<template>
  <KitSection title="CaptureField">
    <KitState label="Field">
      <CaptureField :variant="CaptureFieldVariant.Field">B (right)</CaptureField>
    </KitState>
    <KitState label="Field · listening">
      <CaptureField :variant="CaptureFieldVariant.Field" listening>
        Press a button (5)
      </CaptureField>
    </KitState>
    <KitState label="Quiet">
      <CaptureField :variant="CaptureFieldVariant.Quiet">Change</CaptureField>
    </KitState>
    <KitState label="Quiet · listening">
      <CaptureField :variant="CaptureFieldVariant.Quiet" listening>Change</CaptureField>
    </KitState>
    <template #review>
      <KitIssue title="Field è alto 30: né 28 né 32" :status="IssueStatus.Proposed">
        Field usa il token capture (30), scritto in attesa di Cowork; Quiet è 28, i pulsanti e i
        campi vicini 28 o 32. Nella tabella dei tasti (RemapTableRow) è l'unico controllo da 30.
        <template #current>
          <div class="flex items-center gap-8">
            <CaptureField :variant="CaptureFieldVariant.Field">B (right)</CaptureField>
            <Button>32</Button>
            <Button :size="ButtonSize.Small">28</Button>
          </div>
        </template>
        <template #proposed>
          <div class="flex items-center gap-8">
            <CaptureField :variant="CaptureFieldVariant.Field" class="h-control">
              B (right)
            </CaptureField>
            <Button>32</Button>
            <Button :size="ButtonSize.Small">28</Button>
          </div>
        </template>
        <template #solution>
          Field alto h-control (32) in capture-field/variants.ts, come Select e Input; il token
          capture esce da spacing.css.
        </template>
      </KitIssue>
      <KitIssue title="Quiet in ascolto: col controller il tratteggio non si vede">
        Il tratteggio in accento di Quiet in ascolto è un outline, e mentre il campo ha il focus
        l'anello di focus (styles.css) lo sostituisce. Si preme «Cambia» col controller o con Invio,
        quindi il campo ha sempre il focus mentre ascolta: resta solo il testo in accento a dire che
        aspetta un tasto, e la parola resta «Cambia».
        <template #solution>
          Il segno «in ascolto» di Quiet diventa un bordo tratteggiato (come Field) invece di un
          outline, e il testo dice cosa fare («Premi un tasto», dalla schermata che lo usa,
          ControlsShortcuts).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
