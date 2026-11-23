<script setup lang="ts">
// E03, nothing saved yet: what saves itself and how, «Apri la libreria», «Importa salvataggi…»
// and the formats an import reads.
import { useI18n } from 'vue-i18n'
import SavesEmpty from '@/components/saves/SavesEmpty.vue'
import { emptyTextKey } from '@/components/saves/savesModel'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'

const { t } = useI18n()
const text = (autoq: string) => t(emptyTextKey(autoq), { combo: 'L3 + R1 o F5' })
</script>

<template>
  <KitSection title="Saves empty" :group="KitGroup.Settings">
    <KitState class="w-full" label="Auto quick save every 5 minutes (default)">
      <div class="w-full rounded-card border border-line">
        <SavesEmpty :text="text('5min')" />
      </div>
    </KitState>
    <KitState class="w-full" label="Auto quick save off">
      <div class="w-full rounded-card border border-line">
        <SavesEmpty :text="text('never')" />
      </div>
    </KitState>
  </KitSection>
</template>
