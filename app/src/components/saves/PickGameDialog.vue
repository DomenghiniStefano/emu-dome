<script setup lang="ts">
// V05 «Scegli un altro gioco…»: every library game in title order, the current target chosen
// (and focused first), «Annulla». Controller / keyboard: FlowDialog's own nav scope.
import { useI18n } from 'vue-i18n'
import type { LibraryGame } from '@/emu'
import { Button, ButtonSize } from '@/design-system/components/button'
import FlowDialog from '@/components/game/FlowDialog.vue'
import SavesGameRow from './SavesGameRow.vue'

defineProps<{ games: LibraryGame[]; chosen: LibraryGame | null }>()
const emit = defineEmits<{ choose: [LibraryGame]; cancel: [] }>()

const { t } = useI18n()
</script>

<template>
  <FlowDialog :title="t('saves.importer.pickTitle')" escape-closes @back="emit('cancel')">
    <div class="flex max-h-[50vh] flex-col gap-2 overflow-auto">
      <SavesGameRow
        v-for="(g, i) in games"
        :key="g.id"
        :game="g"
        :detail="t(`console.${g.console}`)"
        :selected="g.id === chosen?.id"
        :data-nav-initial="(chosen ? g.id === chosen.id : i === 0) ? '' : undefined"
        @click="emit('choose', g)"
      />
    </div>
    <template #actions>
      <Button :size="ButtonSize.Medium" @click="emit('cancel')">
        {{ t('saves.importer.cancel') }}
      </Button>
    </template>
  </FlowDialog>
</template>
