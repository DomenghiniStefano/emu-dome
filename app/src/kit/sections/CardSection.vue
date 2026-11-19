<script setup lang="ts">
import { ref } from 'vue'
import { Card, CardDescription, CardTitle, CardVariant } from '@/components/ui/card'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const chosen = ref('normal')
const picture = ref('generated')
const row = ref('Pocket Quest')
const device = ref('pad')
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
    <KitState label="PictureChoice · selected, not, empty (data-empty)">
      <div class="flex gap-10">
        <Card
          v-for="id in ['generated', 'icon', 'user']"
          :key="id"
          as="button"
          :variant="CardVariant.PictureChoice"
          :selected="picture === id"
          :data-empty="id === 'user' ? '' : undefined"
          role="radio"
          :aria-checked="picture === id"
          class="w-[120px] cursor-pointer gap-8"
          @click="picture = id"
        >
          <CardTitle>{{ id }}</CardTitle>
          <CardDescription>One picture of a choice.</CardDescription>
        </Card>
      </div>
    </KitState>
    <KitState label="ListItem · a list's rows (as button), selected and not">
      <div class="flex w-field flex-col gap-2">
        <Card
          v-for="id in ['Pocket Quest', 'Star Pilot']"
          :key="id"
          as="button"
          :variant="CardVariant.ListItem"
          :selected="row === id"
          @click="row = id"
        >
          <span class="flex min-w-0 flex-1 flex-col gap-2">
            <span class="font-medium">{{ id }}</span>
            <span class="text-caption text-ink-muted">1 quick save · 384 kB</span>
          </span>
        </Card>
      </div>
    </KitState>
    <KitState label="Choice · selected and not">
      <div class="flex gap-10">
        <Card
          v-for="id in ['pad', 'keyboard']"
          :key="id"
          as="button"
          :variant="CardVariant.Choice"
          :selected="device === id"
          role="radio"
          :aria-checked="device === id"
          class="w-field cursor-pointer gap-2"
          @click="device = id"
        >
          <CardTitle>{{ id }}</CardTitle>
          <CardDescription>One device of a few.</CardDescription>
        </Card>
      </div>
    </KitState>
    <KitState label="List · with Row (button, disabled, div)">
      <Card :variant="CardVariant.List" class="w-dialog">
        <Card as="button" :variant="CardVariant.Row" class="gap-16">
          <CardTitle>Remap buttons</CardTitle>
        </Card>
        <Card as="button" disabled :variant="CardVariant.Row" class="gap-16">
          <CardTitle>Sticks and triggers</CardTitle>
        </Card>
        <Card :variant="CardVariant.Row" class="gap-16">
          <CardTitle>Touch controls</CardTitle>
        </Card>
      </Card>
    </KitState>
  </KitSection>
</template>
