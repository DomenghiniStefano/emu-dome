<script setup lang="ts">
import { ref } from 'vue'
import { Card, CardDescription, CardTitle, CardVariant } from '@/components/ui/card'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const chosen = ref('normal')
const row = ref('Pocket Quest')
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
  </KitSection>
</template>
