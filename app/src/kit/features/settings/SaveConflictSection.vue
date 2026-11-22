<script setup lang="ts">
// A save conflict (V06 / G04): the two sides as ConflictTwoUp lays them out, one with a picture
// and its play time, the newer without; and the whole dialog, opened from a button.
import { ref } from 'vue'
import ConflictSide from '@/components/saves/ConflictSide.vue'
import ConflictTwoUp from '@/components/saves/ConflictTwoUp.vue'
import { Button } from '@/components/ui/button'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import { CONFLICT } from './mock'

const open = ref(false)
</script>

<template>
  <KitSection title="Save conflict" :group="KitGroup.Settings">
    <KitState label="Two sides (dialog width): older with picture, newer without">
      <div class="grid w-dialog max-w-full grid-cols-2 gap-16">
        <ConflictSide :version="CONFLICT[0]" :newest="false" />
        <ConflictSide :version="CONFLICT[1]" newest />
      </div>
    </KitState>
    <KitState label="The dialog (FlowDialog): B / Esc = «Decidi dopo»">
      <Button @click="open = true">Apri il conflitto</Button>
      <ConflictTwoUp
        v-if="open"
        title="Pocket Quest"
        :versions="CONFLICT"
        @keep="open = false"
        @keep-both="open = false"
        @later="open = false"
      />
    </KitState>

    <template #review>
      <KitIssue
        title="I due lati hanno righe diverse, i pulsanti non si allineano"
        :status="IssueStatus.Open"
      >
        Il lato senza tempo di gioco ha una riga in meno: «Tieni questa» sta più in alto che
        nell’altro e sotto resta uno spazio vuoto. Chi confronta i due lati legge «Tempo di gioco»
        solo da una parte e non sa se dall’altra è zero o sconosciuto.
        <template #solution>
          in ConflictSide.vue la riga «Tempo di gioco» sempre presente («non noto» quando manca,
          testo nuovo in en.json e it.json), e il pulsante al fondo con `mt-auto` (vedi la proposta
          dello slot qui sopra).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
