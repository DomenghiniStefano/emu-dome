<script setup lang="ts">
import { CaptureField, CaptureFieldVariant } from '@/components/ui/capture-field'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
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
  </KitSection>
</template>
