<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Heading, HeadingVariant } from '@/components/ui/heading'
import { LabelledBlock } from '@/components/ui/labelled-block'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
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
  </KitSection>
</template>
