<script setup lang="ts">
import { Card } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { SectionPage, SectionWidth } from '@/components/ui/section-page'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
</script>

<template>
  <KitSection title="SectionPage">
    <KitState label="Page width: the blocks stop at 920 (here the box is narrower), scrolls itself">
      <div class="flex h-field w-dialog max-w-full border border-dashed border-line">
        <SectionPage :width="SectionWidth.Page">
          <Heading>Video</Heading>
          <Card>One block</Card>
          <Card>Another block</Card>
        </SectionPage>
      </div>
    </KitState>
    <KitState label="On the phone: one step all round">
      <div
        data-density="touch"
        class="flex h-field w-dialog max-w-full border border-dashed border-line"
      >
        <SectionPage>
          <Heading>Saves</Heading>
          <Card>One block</Card>
        </SectionPage>
      </div>
    </KitState>
    <KitState label="No scroll of its own: grows inside its parent's scroller (a game's details)">
      <div
        class="flex h-field w-dialog max-w-full flex-col overflow-y-auto border border-dashed border-line"
      >
        <SectionPage :width="SectionWidth.Medium" :scroll="false">
          <Card>A card of the details</Card>
          <Card>Another card</Card>
        </SectionPage>
      </div>
    </KitState>
    <template #review>
      <KitIssue title="Su una finestra larga i blocchi restano a sinistra">
        Con Page, Medium e Wide ogni blocco si ferma alla larghezza massima (920, 1040, 1180) ma
        resta allineato a sinistra: su una finestra da 1920 le Impostazioni e i Controlli lasciano
        una fascia vuota di diverse centinaia di pixel a destra dei blocchi. Non si vede qui (il
        riquadro è più stretto di 920): si vede nell'app a finestra intera.
        <template #solution>
          Una delle due, da decidere col proprietario: i blocchi centrati (items-center in
          section-page/variants.ts, così il vuoto si divide ai due lati), oppure la larghezza
          massima che cresce con la finestra e la seconda colonna (le card affiancate) che riempie
          il posto.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
