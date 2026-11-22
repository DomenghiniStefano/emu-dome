<script setup lang="ts">
// One game of the phone's compact grid (#29, `library.view` «Griglia» on the phone, mockup
// «Libreria sul telefono» B): the cover without its title (the ROM icon larger, the console's
// short name and «N versioni» on it, a deck behind a family of regional versions, #102), the
// title and when it was played under it. A tap on the cover (a right click too) opens the game's
// details (L04). The game played last, when it has a save to pick up from, has «Riprendi» in the
// cover's corner: an accent square with the play icon. A game that is not reachable (E01) is
// faded.
//
// Focusable parts for the windowed grid (`data-cover-part`, coverGridNav.ts `compactStep`): the
// cover, and «Riprendi» on its tile.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Play } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize, CoverStackLayer, coverStackVariants } from '@/components/cover/variants'
import type { LibraryGame } from '@/emu'
import { gamePlayedLabel } from '@/i18n/labels'
import { COVER_OF } from './useLibraryFocus'

const props = withDefaults(
  defineProps<{
    game: LibraryGame
    /** This tile carries the view's «Riprendi». */
    resume?: boolean
    /** Its family's regional versions (#102): from 2, a deck behind it and the «N versioni» tag. */
    versions?: number
  }>(),
  { resume: false, versions: 0 },
)
const emit = defineEmits<{ details: []; resume: [] }>()

const { t } = useI18n()

const off = computed(() => !props.game.available)
const meta = computed(() => gamePlayedLabel(props.game))
</script>

<template>
  <div class="flex min-w-0 flex-col gap-4" :data-game="game.id">
    <div class="relative rounded-card shadow-card" @contextmenu.prevent="emit('details')">
      <template v-if="versions > 1">
        <div :class="coverStackVariants({ layer: CoverStackLayer.Far })" aria-hidden="true"></div>
        <div :class="coverStackVariants({ layer: CoverStackLayer.Near })" aria-hidden="true"></div>
      </template>
      <GameCover
        :game="game"
        :size="CoverSize.Compact"
        :title="false"
        :versions="versions"
        :missing="off"
      />
      <Button
        :variant="ButtonVariant.Bare"
        :size="ButtonSize.Area"
        class="absolute inset-0 rounded-card"
        v-bind="{ [COVER_OF]: game.id }"
        data-cover-part="cover"
        :aria-label="t('library.openDetails', { title: game.title })"
        @click="emit('details')"
      ></Button>
      <Button
        v-if="resume"
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.ToolIcon"
        class="absolute right-6 bottom-6"
        data-cover-part="play"
        :aria-label="t('library.hero.resume')"
        :title="t('library.hero.resume')"
        @click="emit('resume')"
      >
        <Play :size="16" fill="currentColor" :stroke-width="0" aria-hidden="true" />
      </Button>
    </div>
    <div class="flex min-w-0 flex-col" :class="{ 'opacity-missing-label': off }">
      <div class="truncate text-label font-semibold">{{ game.title }}</div>
      <div class="truncate text-caption text-ink-muted tabular-nums">{{ meta }}</div>
    </div>
  </div>
</template>
