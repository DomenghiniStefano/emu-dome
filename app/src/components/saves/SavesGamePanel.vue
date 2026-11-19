<script setup lang="ts">
// V02, one game's saves: its cover, title and what it has saved, «Dettagli del gioco»; then the
// in-game save («Esporta…»), the quick saves as slot tiles, and the in-game save's backups
// («Ripristina»; E04's note when an unreadable in-game save was set aside).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { GameSaves, LibraryGame, QuickSave } from '../../emu'
import { saveWhenLabel, sizeLabel } from '../../i18n/labels'
import { Button } from '@/components/ui/button'
import GameCover from '../GameCover.vue'
import { CoverSize } from '../cover/variants'
import SlotTile from '../SlotTile.vue'
import SaveBackups from './SaveBackups.vue'
import SavesBlock from './SavesBlock.vue'
import SaveWarning from './SaveWarning.vue'
import { useSlotName } from './useSlotName'

const props = defineProps<{
  game: LibraryGame
  saves: GameSaves
  /** What the game has saved, as the list says it. */
  detail: string
}>()
const emit = defineEmits<{
  openDetails: []
  exportInGame: []
  /** «Gioca da qui» on a quick save. */
  play: [slot: number]
  /** «…» on a quick save: its details (V03). */
  more: [quick: QuickSave, event: Event]
  restore: [name: string]
}>()

const { t } = useI18n()
const slotName = useSlotName()

const inGameLine = computed(() =>
  props.saves.inGame
    ? t('saves.inGameLine', {
        when: saveWhenLabel(props.saves.inGame.modified ?? 0),
        size: sizeLabel(props.saves.inGame.size),
      })
    : t('saves.noInGame'),
)
</script>

<template>
  <section class="flex flex-col gap-18">
    <div class="flex items-center gap-14">
      <GameCover
        :size="CoverSize.Row"
        class="flex-none"
        :game="game"
        :badge="false"
        :title="false"
      />
      <div class="min-w-0 flex-1">
        <h2 class="m-0 text-title">{{ game.title }}</h2>
        <div class="text-ink-muted tabular-nums">
          {{ t('saves.gameMeta', { console: t(`console.${game.console}`), meta: detail }) }}
        </div>
      </div>
      <Button @click="emit('openDetails')">{{ t('saves.details') }}</Button>
    </div>

    <SavesBlock :title="t('saves.inGameTitle')">
      <div class="flex items-center gap-12 rounded-10 border border-line bg-surface px-16 py-10">
        <span class="min-w-0 flex-1 tabular-nums">{{ inGameLine }}</span>
        <Button v-if="saves.inGame" @click="emit('exportInGame')">{{ t('saves.export') }}</Button>
      </div>
    </SavesBlock>

    <SavesBlock :title="t('saves.quickTitle')">
      <!-- Each tile: «Gioca da qui» opens the game on that slot, «…» the other actions (V03).
           The date only: a quick save has no play time of its own (Cowork, #37).
           `outdated`: an older version of its core made it (saves.rs, `is_outdated`). -->
      <div v-if="saves.quick.length" class="grid grid-cols-4 gap-10">
        <SlotTile
          v-for="q in saves.quick"
          :key="q.slot"
          :name="slotName(q.slot)"
          :when="q.savedAt !== null ? saveWhenLabel(q.savedAt) : null"
          :thumb-url="q.thumbUrl"
          :outdated="q.outdated"
          @play="emit('play', q.slot)"
          @more="e => emit('more', q, e)"
        />
      </div>
      <p v-else class="m-0 text-ink-muted">{{ t('saves.noQuick') }}</p>
    </SavesBlock>

    <SavesBlock :title="t('saves.backupsTitle')">
      <!-- E04: the unreadable in-game save was set aside. -->
      <SaveWarning v-if="saves.broken.length">
        {{ t('saves.brokenNote', { file: saves.broken[0] }) }}
      </SaveWarning>
      <SaveBackups
        v-if="saves.backups.length"
        :backups="saves.backups"
        @restore="name => emit('restore', name)"
      />
      <p v-else class="m-0 text-ink-muted">{{ t('saves.noBackups') }}</p>
      <p class="m-0 text-caption text-ink-muted">{{ t('saves.backupNote') }}</p>
    </SavesBlock>
  </section>
</template>
