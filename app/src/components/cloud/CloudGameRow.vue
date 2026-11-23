<script setup lang="ts">
// One game of «Giochi su Google Drive» (Trello #31, mockup option A): cover, title, console and
// size, the state pill (Su entrambi / Solo su Drive / Solo qui / Due versioni) and its action
// (Carica / Scarica / Rimuovi da Drive, its icon alone with the words as its name and tooltip). While
// the game file moves, a bar under its line.
import { computed, type Component } from 'vue'
import { useI18n } from 'vue-i18n'
import { CloudDownload, CloudUpload, Trash2 } from '@lucide/vue'
import type { CloudGame } from '@/emu'
import { Badge, BadgeSize, BadgeVariant } from '@/design-system/components/badge'
import { Button, ButtonSize } from '@/design-system/components/button'
import { Progress } from '@/design-system/components/progress'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import { sizeLabel } from '@/i18n/labels'
import { library } from '@/lib/library'
import {
  coverOf,
  gameAction,
  gameTone,
  progressOf,
  transferShare,
  type GameAction,
  type GameTone,
} from './cloud'
import { cloud } from './cloudState'

const props = defineProps<{ game: CloudGame; busy: boolean }>()
const emit = defineEmits<{ act: [GameAction] }>()
const { t } = useI18n()

const TONE: Record<GameTone, BadgeVariant> = {
  ok: BadgeVariant.Success,
  warn: BadgeVariant.Warning,
  quiet: BadgeVariant.Quiet,
}

/** Each row's action as its icon alone (#104: repeated on every game). */
const ACTION_ICON: Record<GameAction, Component> = {
  upload: CloudUpload,
  download: CloudDownload,
  remove: Trash2,
}

const own = computed(() => library.games.find(g => g.id === props.game.id))
const cover = computed(() => coverOf(props.game, own.value))
const action = computed(() => gameAction(props.game))
const transfer = computed(() => {
  const up = progressOf(cloud.progress, 'uploading', props.game.id)
  return up ?? progressOf(cloud.progress, 'downloading', props.game.id)
})
</script>

<template>
  <div class="flex items-center gap-8 px-12 py-6 not-first:border-t not-first:border-line">
    <GameCover
      class="flex-none"
      :game="cover"
      :size="CoverSize.Thumb"
      :badge="false"
      :title="false"
      :missing="!game.here"
    />
    <!-- On a narrow window the state and the action go under the title. -->
    <div class="flex min-w-0 flex-1 flex-wrap items-center gap-x-8 gap-y-4">
      <div class="flex min-w-0 flex-1 basis-card-min flex-col gap-2">
        <span class="font-medium wrap-anywhere">{{ game.title }}</span>
        <span class="text-caption text-ink-muted tabular-nums">{{
          t('cloud.games.meta', {
            console: t(`console.${game.console}`),
            size: sizeLabel(game.size),
          })
        }}</span>
        <Progress
          v-if="busy && transfer"
          class="w-progress max-w-full"
          :value="transferShare(transfer)"
          :label="game.title"
        />
      </div>
      <div class="flex min-w-0 flex-wrap items-center gap-x-8 gap-y-4">
        <Badge :variant="TONE[gameTone(game.state)]" :size="BadgeSize.Pill">
          {{ t(`cloud.games.state.${game.state}`) }}
        </Badge>
        <Button
          v-if="action"
          :size="ButtonSize.ToolIcon"
          :icon="ACTION_ICON[action]"
          :label="t(`cloud.games.${action}`)"
          :disabled="busy"
          @click="emit('act', action)"
        />
      </div>
    </div>
  </div>
</template>
