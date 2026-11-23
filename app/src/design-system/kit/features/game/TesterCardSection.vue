<script setup lang="ts">
// «Prova i pulsanti» (C01, C02): the 12-button tester, the held buttons lit, and what the last
// press did; on C02 the capture's hint above it.
import TesterCard from '@/components/controls/TesterCard.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'
import { TESTER_CELLS } from './mock'

const NONE = new Set<string>()
const HELD = new Set(['A', 'R1', '→'])
</script>

<template>
  <KitSection title="Tester card" :group="KitGroup.Game">
    <KitState label="Idle: nothing pressed yet">
      <div class="w-menu">
        <TesterCard :cells="TESTER_CELLS" :held="NONE" last="" idle="Nessun pulsante premuto" />
      </div>
    </KitState>
    <KitState label="Three held, the last press named">
      <div class="w-menu">
        <TesterCard
          :cells="TESTER_CELLS"
          :held="HELD"
          last="Ultimo pulsante premuto: A · nel gioco: A"
          idle="Nessun pulsante premuto"
        />
      </div>
    </KitState>
    <KitState label="C02: the capture's hint above">
      <div class="w-menu">
        <TesterCard
          hint="Premi il pulsante del controller che vuoi usare per «B»."
          :cells="TESTER_CELLS"
          :held="NONE"
          last=""
          idle="Nessun pulsante premuto"
        />
      </div>
    </KitState>
  </KitSection>
</template>
