<script setup lang="ts">
import { TriangleAlert } from '@lucide/vue'
import { Badge, BadgeSize, BadgeVariant } from '@/design-system/components/badge'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus } from '@/design-system/kit/review'
</script>

<template>
  <KitSection title="Badge">
    <KitState label="Tag: Neutral, Warning, Reserved">
      <div class="flex items-center gap-8">
        <Badge>Newest</Badge>
        <Badge :variant="BadgeVariant.Warning">Save unreadable</Badge>
        <Badge :variant="BadgeVariant.Reserved">Coming soon</Badge>
      </div>
    </KitState>
    <KitState label="Pill: Success, Quiet">
      <div class="flex items-center gap-8">
        <Badge :variant="BadgeVariant.Success" :size="BadgeSize.Pill">In use</Badge>
        <Badge :variant="BadgeVariant.Quiet" :size="BadgeSize.Pill">crt.slangp</Badge>
      </div>
    </KitState>
    <KitState label="Count: one digit, two digits">
      <div class="flex items-center gap-8">
        <Badge :variant="BadgeVariant.Count" :size="BadgeSize.Count">2</Badge>
        <Badge :variant="BadgeVariant.Count" :size="BadgeSize.Count">99</Badge>
      </div>
    </KitState>
    <template #review>
      <KitIssue title="Warning ha lo stesso ambra dell'accento" :status="IssueStatus.Proposed">
        Il token warning in Scuro è F5A524, l'accento predefinito FAA40F; in Chiaro 9A5800 contro
        966002. Con l'accento predefinito un badge «Salvataggio illeggibile» si legge come un
        elemento in accento, che per regola sta solo sull'azione principale. Lo stesso per il titolo
        di DialogTone.Warning e l'icona di ClosingAsk «lento». Con gli altri quattro accenti la
        differenza c'è.
        <template #current>
          <Badge :variant="BadgeVariant.Warning">Save unreadable</Badge>
          <span class="text-label text-accent-text">Testo in accento</span>
        </template>
        <template #proposed>
          <Badge :variant="BadgeVariant.Warning" class="inline-flex items-center gap-4">
            <TriangleAlert :size="12" :stroke-width="2" aria-hidden="true" />
            Save unreadable
          </Badge>
          <span class="text-label text-accent-text">Testo in accento</span>
        </template>
        <template #solution>
          Come nella proposta, BadgeVariant.Warning porta sempre l'icona TriangleAlert (Badge.vue),
          così il significato non dipende dal colore; oppure il token warning si sposta verso
          l'arancio-rosso in colors.css (decisione di colore del proprietario).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
