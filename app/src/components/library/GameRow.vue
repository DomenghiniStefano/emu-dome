<script setup lang="ts">
// One game of the library's list (#29, the phone's default view, mockup «Libreria sul telefono»
// A): a small cover, the title on one line, the console's short name, «N versioni» for a family
// of regional versions (#102) and when it was played, then its button at the right: «Riprendi»
// (the view's main action, accent) on the game played last when it has a save to pick up from,
// «Gioca» on the others. A tap on the row (a right click too) opens the game's details (L04).
// A game that is not reachable (E01) is faded, and its button says why it cannot start.
//
// The row's click area lies under its content (which lets the taps through) and under the button,
// so the controller moves between the row and its button (`data-cover-part`, coverGridNav.ts
// `listStep`) as the windowed list expects.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Badge } from '@/components/ui/badge'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import type { LibraryGame } from '@/emu'
import { gamePlayedLabel } from '@/i18n/labels'
import { COVER_OF } from './useLibraryFocus'

const props = withDefaults(
  defineProps<{
    game: LibraryGame
    /** This row carries the view's «Riprendi». */
    resume?: boolean
    /** Its family's regional versions (#102): from 2, the «N versioni» tag. */
    versions?: number
  }>(),
  { resume: false, versions: 0 },
)
const emit = defineEmits<{ details: []; play: []; resume: [] }>()

const { t } = useI18n()

const off = computed(() => !props.game.available)
const meta = computed(() => gamePlayedLabel(props.game))
</script>

<template>
  <div
    class="relative flex min-w-0 items-center gap-12 border-b border-line py-8"
    :data-game="game.id"
    @contextmenu.prevent="emit('details')"
  >
    <Button
      :variant="ButtonVariant.Bare"
      :size="ButtonSize.Area"
      class="absolute inset-0 rounded-none"
      v-bind="{ [COVER_OF]: game.id }"
      data-cover-part="cover"
      :aria-label="t('library.openDetails', { title: game.title })"
      @click="emit('details')"
    ></Button>
    <GameCover
      class="pointer-events-none"
      :game="game"
      :size="CoverSize.ListRow"
      :badge="false"
      :title="false"
      :missing="off"
    />
    <div
      class="pointer-events-none relative flex min-w-0 flex-1 flex-col gap-2"
      :class="{ 'opacity-missing-label': off }"
    >
      <div class="truncate font-semibold">{{ game.title }}</div>
      <div class="flex min-w-0 items-center gap-6 text-caption text-ink-muted tabular-nums">
        <Badge>{{ t(`consoleShort.${game.console}`) }}</Badge>
        <Badge v-if="versions > 1">{{
          t('library.versions.tag', { n: versions }, versions)
        }}</Badge>
        <span class="min-w-0 truncate">{{ meta }}</span>
      </div>
    </div>
    <Button
      :variant="resume ? ButtonVariant.Primary : ButtonVariant.Soft"
      :size="ButtonSize.Tool"
      class="relative"
      data-cover-part="play"
      :aria-disabled="off || undefined"
      @click="resume ? emit('resume') : emit('play')"
    >
      {{ t(resume ? 'library.hero.resume' : 'library.hero.play') }}
    </Button>
  </div>
</template>
