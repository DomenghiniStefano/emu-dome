<script setup lang="ts">
import { Card } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { SectionPage, SectionWidth } from '@/components/ui/section-page'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
</script>

<template>
  <KitSection title="SectionPage">
    <KitState label="Page width: the blocks stop at 920 (here the box is narrower), scrolls itself">
      <div class="flex h-field w-dialog border border-dashed border-line">
        <SectionPage :width="SectionWidth.Page">
          <Heading>Video</Heading>
          <Card>One block</Card>
          <Card>Another block</Card>
        </SectionPage>
      </div>
    </KitState>
    <KitState label="On the phone: one step all round">
      <div data-density="touch" class="flex h-field w-dialog border border-dashed border-line">
        <SectionPage>
          <Heading>Saves</Heading>
          <Card>One block</Card>
        </SectionPage>
      </div>
    </KitState>
  </KitSection>
</template>
