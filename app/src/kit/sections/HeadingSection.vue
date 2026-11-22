<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Heading, HeadingVariant } from '@/components/ui/heading'
import { LabelledBlock } from '@/components/ui/labelled-block'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'
</script>

<template>
  <KitSection title="Heading">
    <KitState label="Overline: desktop, and on the phone (one step larger)">
      <div class="flex gap-24">
        <Heading :variant="HeadingVariant.Overline">Quick saves</Heading>
        <div data-density="touch">
          <Heading :variant="HeadingVariant.Overline">Quick saves</Heading>
        </div>
      </div>
    </KitState>
    <KitState label="Heading">
      <Heading :variant="HeadingVariant.Heading">Results for “crt”</Heading>
    </KitState>
    <KitState label="Title (the default)">
      <Heading>Your games</Heading>
    </KitState>
    <KitState label="PageTitle (a quick menu page)">
      <Heading :variant="HeadingVariant.PageTitle">Restart the game?</Heading>
    </KitState>
    <KitState label="Display: desktop, and on the phone">
      <div class="flex items-end gap-24">
        <Heading :variant="HeadingVariant.Display">Welcome</Heading>
        <div data-density="touch">
          <Heading :variant="HeadingVariant.Display">Welcome</Heading>
        </div>
      </div>
    </KitState>
    <KitState label="Hero: a game's name beside its cover, desktop and phone">
      <div class="flex items-end gap-24">
        <Heading :variant="HeadingVariant.Hero">Pocket Quest</Heading>
        <div data-density="touch">
          <Heading :variant="HeadingVariant.Hero">Pocket Quest</Heading>
        </div>
      </div>
    </KitState>
    <KitState label="LabelledBlock: its label, an aside at the far end, the block">
      <LabelledBlock title="Quick saves" class="w-dialog">
        <template #aside>
          <Button :variant="ButtonVariant.Link" :size="ButtonSize.LabelLink">More slots</Button>
        </template>
        <Card>The block</Card>
      </LabelledBlock>
    </KitState>
    <template #review>
      <KitIssue title="Quattro titoli a 1–2 px di distanza" :status="IssueStatus.Proposed">
        Heading è 15/20, PageTitle (il titolo di una pagina del menu rapido) 17/22, il titolo di un
        dialogo 18/24, Title 20/26. PageTitle e il titolo dei dialoghi si vedono uno dopo l'altro
        (una pagina del menu rapido, poi la sua domanda) e differiscono di 1 px: non si leggono come
        due livelli, solo come una svista.
        <template #current>
          <div class="flex flex-col gap-8">
            <Heading :variant="HeadingVariant.PageTitle">Riavvia? (PageTitle, 17)</Heading>
            <span class="text-dialog text-ink">Riavvia? (dialogo, 18)</span>
          </div>
        </template>
        <template #proposed>
          <div class="flex flex-col gap-8">
            <Heading :variant="HeadingVariant.PageTitle" class="text-dialog">
              Riavvia? (PageTitle, 18)
            </Heading>
            <span class="text-dialog text-ink">Riavvia? (dialogo, 18)</span>
          </div>
        </template>
        <template #solution>
          PageTitle prende il passo dialog (18/24) in heading/variants.ts; il token in sospeso
          page-title esce da typography.css.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
