<script setup lang="ts">
// L05's side card: the in-game save (last change, backups; a set-aside unreadable one, E04) with
// «Mostra i backup» (the Saves section on this game), «Importa…» (V05 for this game) and
// «Esporta…». Phone in portrait: the last change in one line, no export.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { History as HistoryIcon, Import, Share, TriangleAlert } from '@lucide/vue'
import { emu, type GameSaves, type LibraryGame } from '@/emu'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import { saveWhenLabel } from '@/i18n/labels'
import DetailsAction from './DetailsAction.vue'
import DetailsCard from './DetailsCard.vue'
import { Heading, HeadingVariant } from '@/design-system/components/heading'

const props = defineProps<{ game: LibraryGame; saves: GameSaves | null; phone: boolean }>()
const emit = defineEmits<{
  backups: []
  import: []
}>()

const { t } = useI18n()
const toasts = useToasts()
const failWith = useErrorSink()

const lastChange = computed(() => {
  const g = props.saves?.inGame
  return g ? saveWhenLabel(g.modified ?? 0) : null
})
const backupsLine = computed(() => {
  const list = props.saves?.backups ?? []
  const oldest = list[list.length - 1]
  return oldest
    ? t(
        'details.saves.backupsLine',
        { n: list.length, when: saveWhenLabel(oldest.at) },
        list.length,
      )
    : t('saves.noBackups')
})

async function exportInGame() {
  try {
    if (await emu.saves.export(props.game.id, { kind: 'inGame' }))
      toasts.notify(t('saves.exported'))
  } catch (e) {
    failWith(e)
  }
}
</script>

<template>
  <!-- The phone's card text is 14px (spec Decision 5, design); its buttons take it. -->
  <DetailsCard as="aside" class="gap-8 touch:gap-6 touch:text-cta-inline">
    <Heading as="h2" :variant="HeadingVariant.Overline">{{ t('saves.inGameTitle') }}</Heading>
    <div v-if="phone" class="tabular-nums">
      {{
        lastChange ? t('details.saves.lastChangeLine', { when: lastChange }) : t('saves.noInGame')
      }}
    </div>
    <template v-else>
      <div class="text-ink-muted">{{ t('details.saves.inGameLead') }}</div>
      <div class="flex justify-between gap-12 tabular-nums">
        <span class="flex-none">{{ t('details.saves.lastChange') }}</span>
        <span class="text-right text-ink-muted">{{ lastChange ?? t('saves.noInGame') }}</span>
      </div>
      <div class="flex justify-between gap-12">
        <span class="flex-none">{{ t('details.saves.backups') }}</span>
        <span class="text-right text-ink-muted tabular-nums">{{ backupsLine }}</span>
      </div>
    </template>
    <!-- E04: the unreadable in-game save was set aside. -->
    <div v-if="saves?.broken.length" class="flex gap-8 text-caption text-warning">
      <TriangleAlert :size="16" :stroke-width="1.75" aria-hidden="true" />
      <span>{{ t('saves.brokenNote', { file: saves.broken[0] }) }}</span>
    </div>
    <div class="mt-4 flex flex-wrap gap-8 touch:mt-0">
      <DetailsAction :icon="HistoryIcon" @click="emit('backups')">
        {{ t('details.saves.showBackups') }}
      </DetailsAction>
      <DetailsAction :icon="Import" @click="emit('import')">{{ t('saves.import') }}</DetailsAction>
      <DetailsAction v-if="!phone && saves?.inGame" :icon="Share" @click="exportInGame">
        {{ t('saves.export') }}
      </DetailsAction>
    </div>
  </DetailsCard>
</template>
