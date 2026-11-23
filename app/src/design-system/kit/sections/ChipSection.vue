<script setup lang="ts">
import { ref } from 'vue'
import { Chip, ChipVariant } from '@/design-system/components/chip'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus } from '@/design-system/kit/review'

const filter = ref('all')
const options = ref(['png'])
const toggle = (id: string) =>
  (options.value = options.value.includes(id)
    ? options.value.filter(o => o !== id)
    : [...options.value, id])
</script>

<template>
  <KitSection title="Chip">
    <KitState label="Filter · selected / not (aria-pressed)">
      <div class="flex gap-8">
        <Chip
          v-for="id in ['all', 'gba', 'nds']"
          :key="id"
          :selected="filter === id"
          :aria-pressed="filter === id"
          @click="filter = id"
        >
          {{ id }}
        </Chip>
      </div>
    </KitState>
    <KitState label="Option · selected / not (checkbox)">
      <div class="flex gap-8">
        <Chip
          v-for="id in ['png', 'bmp']"
          :key="id"
          :variant="ChipVariant.Option"
          :selected="options.includes(id)"
          role="checkbox"
          :aria-checked="options.includes(id)"
          @click="toggle(id)"
        >
          {{ id }}
        </Chip>
      </div>
    </KitState>
    <KitState label="Option · shown only (as span)">
      <Chip as="span" :variant="ChipVariant.Option">1 png</Chip>
    </KitState>
    <KitState label="Filter · disabled (looks the same: the row that holds it fades)">
      <Chip disabled>3ds</Chip>
    </KitState>
    <template #review>
      <KitIssue title="«Scelto» con due bordi diversi" :status="IssueStatus.Proposed">
        Filter scelto ha il bordo accent-edge (l'accento al 40%), Option scelto il bordo
        accent-text. In Chiaro il bordo di Filter contro il bianco è 1,33:1: il filtro scelto si
        riconosce solo dal fondo tenue. In più Filter si schiarisce sotto il mouse e Option no.
        <template #current>
          <Chip selected :aria-pressed="true">gba</Chip>
          <Chip :variant="ChipVariant.Option" selected role="checkbox" :aria-checked="true">
            png
          </Chip>
        </template>
        <template #proposed>
          <Chip selected :aria-pressed="true" class="data-selected:border-accent-text">gba</Chip>
          <Chip :variant="ChipVariant.Option" selected role="checkbox" :aria-checked="true">
            png
          </Chip>
        </template>
        <template #solution>
          In chip/variants.ts un solo bordo per «scelto» (accent-text, nella base) e lo stesso hover
          sulle due varianti (le pillole delle Tabs sul telefono usano lo stesso accent-edge: da
          allineare insieme).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
