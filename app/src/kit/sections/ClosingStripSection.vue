<script setup lang="ts">
// What shows while the app closes with a game open (#86): the strip while saving and once saved,
// and the card it grows into when saving is slow or fails. The cards register their controller
// scope here too (as in the app): the arrows move between their buttons.
import ClosingAsk from '@/components/closing/ClosingAsk.vue'
import ClosingStrip from '@/components/closing/ClosingStrip.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'

const diskFull = { code: 'saveFileFailed', params: { detail: 'disk full' } } as const
</script>

<template>
  <KitSection title="Closing strip">
    <KitState label="Saving: the icon pulses (still with reduced motion)">
      <ClosingStrip :saved="false" />
    </KitState>
    <KitState label="Saved: held 600 ms, then the app exits">
      <ClosingStrip :saved="true" />
    </KitState>
    <KitState label="Slow (10 s): the strip grows into a card, focus on Wait">
      <ClosingAsk :ask="{ kind: 'timeout' }" :focus="false" />
    </KitState>
    <KitState label="A save failed: what failed, close is the only way on">
      <ClosingAsk :ask="{ kind: 'failed', error: diskFull }" :focus="false" />
    </KitState>
  </KitSection>
</template>
