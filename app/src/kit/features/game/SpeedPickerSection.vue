<script setup lang="ts">
// The quick menu's speeds (G07): Normale, Avanti veloce, Rallentatore with their factors, and the
// link to G16.
import { ref } from 'vue'
import type { Speed } from '@/emu'
import SpeedPicker from '@/components/quick-menu/SpeedPicker.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import GameStage from './GameStage.vue'
import { FF_4, MAX, NORMAL, SPEED_CHOICES } from './mock'

const speed = ref<Speed>(NORMAL)
</script>

<template>
  <KitSection title="Speed picker" :group="KitGroup.Game">
    <KitState label="Normal, then click: fast forward 4×, slow motion 0.5×">
      <GameStage panel>
        <SpeedPicker
          :choices="SPEED_CHOICES"
          :speed="speed"
          ff-option="4"
          slow-option="0.5"
          @speed="s => (speed = s)"
        />
      </GameStage>
    </KitState>
    <KitState label="Fast forward chosen">
      <GameStage panel>
        <SpeedPicker :choices="SPEED_CHOICES" :speed="FF_4" ff-option="4" slow-option="0.5" />
      </GameStage>
    </KitState>
    <KitState label="A speed set elsewhere (Max from a shortcut): nothing chosen">
      <GameStage panel>
        <SpeedPicker :choices="SPEED_CHOICES" :speed="MAX" ff-option="4" slow-option="0.5" />
      </GameStage>
    </KitState>

    <template #review>
      <KitIssue
        title="Con una velocità scelta altrove nessuna voce è accesa"
        :status="IssueStatus.Open"
      >
        Se il gioco va a una velocità che non è una delle tre (il codice lo prevede:
        <code>chosenSpeed</code> restituisce '' per «una velocità impostata altrove»; qui Max con
        l’avanti veloce a 4×, terzo stato), il selettore non ha nessuna voce scelta e non dice a che
        velocità si sta giocando. Chi apre il menu per «tornare normale» non vede da dove parte; col
        controller il focus entra su Normale senza un segno della velocità attuale.
        <template #solution>
          da decidere con il proprietario: una quarta voce temporanea con la velocità attuale
          («Max») finché resta quella, oppure una riga sotto il selettore («Ora: Max») in
          <code>text-caption text-ink-muted</code>. In SpeedPicker.vue, quando
          <code>chosenSpeed</code> è vuoto.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
