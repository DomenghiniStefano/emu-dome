<script setup lang="ts">
import { CaptureField, CaptureFieldVariant } from '@/design-system/components/capture-field'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
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
