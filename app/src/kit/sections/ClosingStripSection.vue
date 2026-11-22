<script setup lang="ts">
// What shows while the app closes with a game open (#86): the strip while saving and once saved,
// and the card it grows into when saving is slow or fails. The cards register their controller
// scope here too (as in the app): the arrows move between their buttons.
import { CloudUpload } from '@lucide/vue'
import ClosingAsk from '@/components/closing/ClosingAsk.vue'
import ClosingStrip from '@/components/closing/ClosingStrip.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'

const diskFull = { code: 'saveFileFailed', params: { detail: 'disk full' } } as const
// CloudPushStrip's words (G14, #31), the longest the strip carries today.
const PUSH = {
  busy: 'Copia dei salvataggi su Google Drive',
  hint: '· puoi continuare',
  done: 'Salvataggi copiati su Google Drive',
}
</script>

<template>
  <KitSection title="Closing strip">
    <KitState label="Saving: the icon pulses (still with reduced motion)">
      <ClosingStrip :saved="false" />
    </KitState>
    <KitState label="Saved: held 600 ms, then the app exits">
      <ClosingStrip :saved="true" />
    </KitState>
    <KitState label="The same strip for a copy to Google Drive (CloudPushStrip, G14)">
      <ClosingStrip :saved="false" :icon="CloudUpload" v-bind="PUSH" />
    </KitState>
    <KitState label="Slow (10 s): the strip grows into a card, focus on Wait">
      <ClosingAsk :ask="{ kind: 'timeout' }" :focus="false" />
    </KitState>
    <KitState label="A save failed: what failed, close is the only way on">
      <ClosingAsk :ask="{ kind: 'failed', error: diskFull }" :focus="false" />
    </KitState>
    <template #review>
      <KitIssue
        title="Il testo non va a capo: su un telefono esce dalla striscia"
        :status="IssueStatus.Proposed"
      >
        Il testo della striscia è su una riga sola (whitespace-nowrap). Con le parole di Google
        Drive («Copia dei salvataggi su Google Drive · puoi continuare») la striscia supera i 360 px
        di un telefono stretto meno i suoi margini: il testo esce dal vetro o dalla finestra. Qui
        sotto la striscia è in un riquadro largo 320.
        <template #current>
          <div class="flex w-menu max-w-full border border-dashed border-line-strong">
            <ClosingStrip :saved="false" :icon="CloudUpload" v-bind="PUSH" />
          </div>
        </template>
        <template #proposed>
          <div class="flex w-menu max-w-full border border-dashed border-line-strong">
            <ClosingStrip
              :saved="false"
              :icon="CloudUpload"
              v-bind="PUSH"
              class="min-w-0 [&_p]:flex-wrap [&_p]:whitespace-normal"
            />
          </div>
        </template>
        <template #solution>
          In ClosingStrip.vue il testo va a capo (il suggerimento scende sotto la frase) e la
          striscia si stringe nel suo contenitore; su desktop resta su una riga.
        </template>
      </KitIssue>
      <KitIssue title="«Lento» ha il cerchio tinto, «fallito» no">
        Nella card di ClosingAsk l'icona del caso lento sta su un cerchio warning-soft, quella del
        salvataggio fallito (il caso più grave) su un cerchio surface-raised neutro, perché non
        esiste un danger-soft. Il caso grave sembra il meno colorato dei due.
        <template #solution>
          Un token danger-soft (14% di danger, come warning-soft e success-soft) in colors.css,
          usato dal cerchio del caso fallito in ClosingAsk.vue.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
