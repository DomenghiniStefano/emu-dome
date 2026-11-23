<script setup lang="ts">
// «Giochi, dal più pesante» (S1 / S2, the owner's answer A): the cover, the title, the square
// console tag, the size, and where the game is: the square «SU DRIVE» tag for a game EmuDome
// downloaded that Drive has too («Togli dal telefono»), «In <cartella>» for a game of the user's
// folders («Apri la cartella» where the system can show it). EmuDome never deletes a user's
// file. The actions are the icon alone, their words as name and tooltip (#104: repeated on
// every row).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { FolderOpen, Trash2 } from '@lucide/vue'
import type { StorageGame } from '@/emu'
import { Badge, BadgeVariant } from '@/design-system/components/badge'
import { Button, ButtonSize } from '@/design-system/components/button'
import GameCover, { type CoverGame } from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import { sizeLabel } from '@/i18n/labels'
import { library } from '@/lib/library'
import type { GameAction } from './storage'

const props = defineProps<{
  game: StorageGame
  action: GameAction
  busy: boolean
  /** «Togli dal telefono», or «… da questo dispositivo» off the phone. */
  removeLabel: string
}>()
const emit = defineEmits<{ remove: []; openFolder: [] }>()
const { t } = useI18n()

const cover = computed<CoverGame>(
  () =>
    library.games.find(g => g.id === props.game.id) ?? {
      console: props.game.console,
      title: props.game.title,
      colour: 'var(--surface-raised)',
      cover: 'generated',
      coverUrl: null,
    },
)
const where = computed(() => {
  const l = props.game.location
  if (l.kind === 'app') return null
  return l.name ? t('storage.page.inFolder', { folder: l.name }) : t('storage.page.inYourFolder')
})
</script>

<template>
  <li class="flex items-center gap-8 px-10 py-6 not-first:border-t not-first:border-line">
    <GameCover
      class="flex-none"
      :game="cover"
      :size="CoverSize.Thumb"
      :badge="false"
      :title="false"
    />
    <div class="flex min-w-0 flex-1 flex-col gap-2">
      <span class="truncate font-medium">{{ game.title }}</span>
      <span class="flex min-w-0 items-center gap-6 text-caption text-ink-muted tabular-nums">
        <Badge>{{ t(`consoleShort.${game.console}`) }}</Badge>
        <span class="flex-none">{{ sizeLabel(game.size) }}</span>
        <Badge v-if="action === 'remove'" :variant="BadgeVariant.Success">
          {{ t('storage.page.onDrive') }}
        </Badge>
        <span v-else-if="where" class="truncate">{{ where }}</span>
      </span>
    </div>
    <Button
      v-if="action === 'remove'"
      :size="ButtonSize.IconRow"
      :icon="Trash2"
      :label="removeLabel"
      :disabled="busy"
      @click="emit('remove')"
    />
    <Button
      v-else-if="action === 'openFolder'"
      :size="ButtonSize.IconRow"
      :icon="FolderOpen"
      :label="t('storage.page.openFolder')"
      @click="emit('openFolder')"
    />
  </li>
</template>
