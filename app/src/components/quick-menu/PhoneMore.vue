<script setup lang="ts">
// The phone quick menu's second level, «Altro» (owner, 2026-11-22): the pages (Salvataggi with the
// quick slot, Velocità with the speed in use, Schermi e immagine, Controlli) and the rarer
// actions, «Riavvia il gioco» last. «Chiudi il gioco» is in the first level's strip (Trello #121).
// The rows are phoneMenu.ts's; what each does is QuickMenu.vue's.
import { useI18n } from 'vue-i18n'
import PageRow from './PageRow.vue'
import type { MoreId, MoreRow } from './phoneMenu'
import { openerMark } from './view'

defineProps<{
  rows: MoreRow[]
  /** The value a row shows before its chevron, by row. */
  values: Partial<Record<MoreId, string>>
}>()
const emit = defineEmits<{ run: [MoreRow, Event] }>()

const { t } = useI18n()
</script>

<template>
  <nav class="flex flex-col gap-2" :aria-label="t('qm.moreLabel')">
    <PageRow
      v-for="(r, i) in rows"
      :key="r.id"
      v-bind="{ ...openerMark(r.id), ...(i === 0 ? { 'data-sub-initial': '' } : {}) }"
      :icon="r.icon"
      :label="t(r.label)"
      :value="values[r.id]"
      :leads="r.page !== undefined"
      :pressed="r.pressed"
      @click="emit('run', r, $event)"
    />
  </nav>
</template>
