<script setup lang="ts">
import { ref } from 'vue'
import { Camera, ChevronRight, Rows2, VolumeX } from '@lucide/vue'
import { Card, CardDescription, CardTitle, CardVariant } from '@/components/ui/card'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const chosen = ref('normal')
const slot = ref(1)
const muted = ref(true)
</script>

<template>
  <KitSection title="Card">
    <KitState label="Surface">
      <Card class="w-field gap-10">
        <CardTitle>Files</CardTitle>
        <CardDescription>The game file and where its saves are.</CardDescription>
      </Card>
    </KitState>
    <KitState label="Raised · a choice (as button), selected and not">
      <div class="flex gap-10">
        <Card
          v-for="id in ['normal', 'wide']"
          :key="id"
          as="button"
          :variant="CardVariant.Raised"
          :selected="chosen === id"
          role="radio"
          :aria-checked="chosen === id"
          class="w-field cursor-pointer gap-6"
          @click="chosen = id"
        >
          <CardTitle>{{ id }}</CardTitle>
          <CardDescription>One of a choice of cards.</CardDescription>
        </Card>
      </div>
    </KitState>
    <KitState label="Glass">
      <Card :variant="CardVariant.Glass" class="w-field gap-10">
        <CardTitle>Quick menu</CardTitle>
        <CardDescription>Thick glass over the haze.</CardDescription>
      </Card>
    </KitState>
    <KitState label="Tile, TileTouch · a slot to choose (as button), chosen and not">
      <div class="flex gap-8">
        <Card
          v-for="n in [1, 2]"
          :key="n"
          as="button"
          :variant="CardVariant.Tile"
          :selected="slot === n"
          class="w-[120px] gap-4"
          @click="slot = n"
        >
          <CardTitle>Slot {{ n }}</CardTitle>
          <CardDescription>18/11/26</CardDescription>
        </Card>
        <Card as="button" :variant="CardVariant.TileTouch" selected class="w-[120px] gap-[3px]">
          <CardTitle>Slot 3</CardTitle>
          <CardDescription>Phone</CardDescription>
        </Card>
      </div>
    </KitState>
    <KitState label="Tool (on, off, disabled), ToolTouch">
      <div class="flex gap-8">
        <Card
          as="button"
          :variant="CardVariant.Tool"
          :selected="muted"
          :aria-pressed="muted"
          class="w-[120px] gap-6"
          @click="muted = !muted"
        >
          <VolumeX :size="20" :stroke-width="1.75" aria-hidden="true" />
          Mute
        </Card>
        <Card as="button" :variant="CardVariant.Tool" class="w-[120px] gap-6">
          <Camera :size="20" :stroke-width="1.75" aria-hidden="true" />
          Screenshot
        </Card>
        <Card as="button" :variant="CardVariant.Tool" disabled class="w-[120px] gap-6">
          <Camera :size="20" :stroke-width="1.75" aria-hidden="true" />
          Not yet
        </Card>
        <Card as="button" :variant="CardVariant.ToolTouch" class="w-[160px] gap-8">
          <Camera :size="18" :stroke-width="1.75" aria-hidden="true" />
          Screenshot
        </Card>
      </div>
    </KitState>
    <KitState label="Row (and disabled), ListRow">
      <div class="flex w-field flex-col gap-8">
        <Card as="button" :variant="CardVariant.Row" class="gap-12">
          <Rows2 :size="18" :stroke-width="1.75" aria-hidden="true" />
          <span class="flex-1">Screens and picture</span>
          <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Card>
        <Card as="button" :variant="CardVariant.Row" disabled class="gap-12">
          <span class="flex-1">Not here</span>
        </Card>
        <Card as="button" :variant="CardVariant.ListRow" class="gap-12">
          <span class="flex-1">Controller shortcuts</span>
          <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Card>
      </div>
    </KitState>
  </KitSection>
</template>
