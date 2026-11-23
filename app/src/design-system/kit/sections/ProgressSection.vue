<script setup lang="ts">
import { Progress } from '@/design-system/components/progress'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
</script>

<template>
  <KitSection title="Progress">
    <KitState label="Indeterminate (no total known: the band sweeps)">
      <Progress class="w-scan max-w-full" label="Scanning" />
    </KitState>
    <KitState label="Determinate: 40%, done">
      <div class="flex w-scan max-w-full flex-col gap-8">
        <Progress :value="40" label="Importing" />
        <Progress :value="100" label="Done" />
      </div>
    </KitState>
    <KitState label="On a card, its text above (a game copying to Google Drive)">
      <div
        class="flex w-scan max-w-full flex-col gap-8 rounded-card border border-line bg-surface p-12"
      >
        <span class="text-caption text-ink-muted tabular-nums">12 MB of 30 MB</span>
        <Progress :value="40" label="Copying, on a card" />
      </div>
    </KitState>
    <template #review>
      <KitIssue title="In Chiaro non si vede quanto manca">
        Il binario è surface-raised senza bordo: in Chiaro contro lo sfondo fa 1,03:1 e contro una
        card bianca 1,08:1. Al 40% si vede la parte fatta ma non la lunghezza totale, quindi non
        quanto manca; con la banda che scorre (indeterminata) sembra una linea che si muove nel
        vuoto. Passa il Kit a Chiaro e guarda gli stati qui sopra. Nessun token di oggi va bene:
        line-strong sarebbe troppo scuro per un fondo pieno, line è troppo chiaro.
        <template #solution>
          Un token «binario» per tema in colors.css (in Chiaro circa il 10–12% di nero, in Scuro e
          Nero l'attuale surface-raised), usato dal fondo di Progress (progress/variants.ts) e dal
          binario dello Slider; il valore lo decide il proprietario.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
