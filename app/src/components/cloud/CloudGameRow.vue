<script setup lang="ts">
// One game of «Giochi su Google Drive» (Trello #31, mockup option A): cover, title, console and
// size, the state pill (Su entrambi / Solo su Drive / Solo qui / Due versioni) and its action
// (Carica / Scarica / Rimuovi da Drive). While the game file moves, a bar under its line.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CloudGame } from '@/emu'
import { Badge, BadgeSize, BadgeVariant } from '@/components/ui/badge'
import { Button, ButtonSize } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
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

const own = computed(() => library.games.find(g => g.id === props.game.id))
const cover = computed(() => coverOf(props.game, own.value))
const action = computed(() => gameAction(props.game))
const transfer = computed(() => {
  const up = progressOf(cloud.progress, 'uploading', props.game.id)
  return up ?? progressOf(cloud.progress, 'downloading', props.game.id)
})
</script>

<template>
  <div class="flex items-center gap-12 px-12 py-8 not-first:border-t not-first:border-line">
    <GameCover
      class="flex-none"
      :game="cover"
      :size="CoverSize.Thumb"
      :badge="false"
      :title="false"
      :missing="!game.here"
    />
    <div class="flex min-w-0 flex-1 flex-col gap-2">
      <span class="font-medium wrap-anywhere">{{ game.title }}</span>
      <span class="text-caption text-ink-muted tabular-nums">{{
        t('cloud.games.meta', { console: t(`console.${game.console}`), size: sizeLabel(game.size) })
      }}</span>
      <Progress
        v-if="busy && transfer"
        class="w-progress max-w-full"
        :value="transferShare(transfer)"
        :label="game.title"
      />
    </div>
    <Badge :variant="TONE[gameTone(game.state)]" :size="BadgeSize.Pill">
      {{ t(`cloud.games.state.${game.state}`) }}
    </Badge>
    <Button v-if="action" :size="ButtonSize.Default" :disabled="busy" @click="emit('act', action)">
      {{ t(`cloud.games.${action}`) }}
    </Button>
  </div>
</template>
