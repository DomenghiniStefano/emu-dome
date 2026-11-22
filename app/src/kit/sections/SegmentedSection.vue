<script setup lang="ts">
import { ref } from 'vue'
import { Segmented, SegmentedItem, SegmentedVariant } from '@/components/ui/segmented'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'

const speed = ref('normal')
const view = ref('list')
const weightNow = ref('normal')
const weightProposed = ref('normal')
const partNow = ref('normal')
const partProposed = ref('normal')
</script>

<template>
  <KitSection title="Segmented">
    <KitState label="One chosen">
      <Segmented v-model="speed" aria-label="Speed">
        <SegmentedItem value="normal">Normal</SegmentedItem>
        <SegmentedItem value="fast">Fast forward</SegmentedItem>
        <SegmentedItem value="slow">Slow motion</SegmentedItem>
      </Segmented>
    </KitState>
    <KitState label="Strong">
      <Segmented v-model="speed" :variant="SegmentedVariant.Strong" aria-label="Speed">
        <SegmentedItem value="normal">Normal</SegmentedItem>
        <SegmentedItem value="fast">Fast forward</SegmentedItem>
      </Segmented>
    </KitState>
    <KitState label="One item disabled">
      <Segmented v-model="speed" aria-label="Speed">
        <SegmentedItem value="normal">Normal</SegmentedItem>
        <SegmentedItem value="fast" disabled>Fast forward</SegmentedItem>
      </Segmented>
    </KitState>
    <KitState label="Disabled">
      <Segmented model-value="normal" disabled aria-label="Speed">
        <SegmentedItem value="normal">Normal</SegmentedItem>
        <SegmentedItem value="fast">Fast forward</SegmentedItem>
      </Segmented>
    </KitState>
    <KitState label="Stacked (a name and its value, one line): desktop, and on the phone">
      <div class="flex w-field flex-col gap-6">
        <div
          v-for="density in ['desktop', 'touch']"
          :key="density"
          :data-density="density"
          class="flex flex-col"
        >
          <Segmented v-model="speed" :variant="SegmentedVariant.Stacked" aria-label="Speed">
            <SegmentedItem v-for="id in ['normal', 'fast', 'slow']" :key="id" :value="id">
              <span class="font-semibold">{{ id }}</span>
              <span class="text-caption">1x</span>
            </SegmentedItem>
          </Segmented>
        </div>
      </div>
    </KitState>
    <KitState label="Bar (a switch on a toolbar, beside its buttons): desktop, and on the phone">
      <div class="flex flex-col gap-6">
        <div
          v-for="density in ['desktop', 'touch']"
          :key="density"
          :data-density="density"
          class="flex"
        >
          <Segmented v-model="view" :variant="SegmentedVariant.Bar" aria-label="View">
            <SegmentedItem value="list">List</SegmentedItem>
            <SegmentedItem value="grid">Grid</SegmentedItem>
          </Segmented>
        </div>
      </div>
    </KitState>
    <template #review>
      <KitIssue
        title="Il segmento scelto ingrassa e sposta gli altri"
        :status="IssueStatus.Proposed"
      >
        In Default il segmento scelto passa da medium a semibold: la parola si allarga di qualche
        pixel e i segmenti alla sua destra si spostano a ogni scelta. Strong esiste per questo
        (tutto semibold), ma è l'unica differenza fra le due varianti. Scegli fra «Normal» e «Fast
        forward» qui a sinistra e guarda «Slow motion».
        <template #current>
          <Segmented v-model="weightNow" aria-label="Weight, now">
            <SegmentedItem value="normal">Normal</SegmentedItem>
            <SegmentedItem value="fast">Fast forward</SegmentedItem>
            <SegmentedItem value="slow">Slow motion</SegmentedItem>
          </Segmented>
        </template>
        <template #proposed>
          <Segmented v-model="weightProposed" aria-label="Weight, proposed">
            <SegmentedItem
              v-for="(name, id) in { normal: 'Normal', fast: 'Fast forward', slow: 'Slow motion' }"
              :key="id"
              :value="id"
              class="data-[state=on]:font-medium"
            >
              {{ name }}
            </SegmentedItem>
          </Segmented>
        </template>
        <template #solution>
          La scelta si vede dal fondo e dal colore, il peso resta medium (segmented/variants.ts);
          Strong (2 usi, ControlsRemap) diventa Default o resta solo se il proprietario vuole i
          segmenti semibold.
        </template>
      </KitIssue>
      <KitIssue
        title="Un segmento disabilitato sembra solo non scelto"
        :status="IssueStatus.Proposed"
      >
        «Fast forward» disabilitato ha lo stesso grigio di un segmento libero; col mouse non
        risponde e col controller il passo lo salta, senza un segno del perché.
        <template #current>
          <Segmented v-model="partNow" aria-label="One disabled, now">
            <SegmentedItem value="normal">Normal</SegmentedItem>
            <SegmentedItem value="fast" disabled>Fast forward</SegmentedItem>
            <SegmentedItem value="slow">Slow motion</SegmentedItem>
          </Segmented>
        </template>
        <template #proposed>
          <Segmented v-model="partProposed" aria-label="One disabled, proposed">
            <SegmentedItem value="normal">Normal</SegmentedItem>
            <SegmentedItem value="fast" disabled class="disabled:opacity-disabled">
              Fast forward
            </SegmentedItem>
            <SegmentedItem value="slow">Slow motion</SegmentedItem>
          </Segmented>
        </template>
        <template #solution>
          disabled:opacity-disabled sui segmenti in segmented/variants.ts (il gruppo intero
          disabilitato: vedi Switch).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
