<script setup lang="ts">
// C02's «Prova i pulsanti» beside the table: what to do (or the capture's hint), the tester, and
// what the last press does in the game.
//
// Off the design system's scales (Cowork): the 12px last-press line on the 18px line.
import { useI18n } from 'vue-i18n'
import { Card } from '@/components/ui/card'
import ButtonTester from '../ButtonTester.vue'

defineProps<{
  hint: string
  cells: { id: string; label: string }[]
  held: Set<string>
  last: string
}>()
const { t } = useI18n()
</script>

<template>
  <Card class="gap-10">
    <div class="text-overline text-ink-muted">{{ t('controls.tester') }}</div>
    <div class="text-ink-muted">{{ hint }}</div>
    <ButtonTester :cells="cells" :held="held" :label="t('controls.overview.outline')" />
    <div class="text-caption leading-[18px] text-ink-muted" aria-live="polite">
      {{ last || t('controls.noPress') }}
    </div>
  </Card>
</template>
