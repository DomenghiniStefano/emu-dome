<script setup lang="ts">
// One game of «Continua a giocare»: the newest quick save's picture (or the game's colour), the
// title and when it was played, «Riprendi» (or «Gioca» with nothing saved) and «Salvataggi»,
// and where «Riprendi» picks up. The first card's «Riprendi» is the view's main action.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Play, Save } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardPad } from '@/components/ui/card'
import { cn } from '@/lib/cn'
import SaveThumb from '@/components/saves/SaveThumb.vue'
import type { LibraryGame } from '@/emu'
import type { ResumePoint } from '@/lib/gameDetails'
import { lastPlayedLabel, playTimeLabel } from '@/i18n/labels'
import { resumeSource } from './libraryView'

const props = defineProps<{ game: LibraryGame; point?: ResumePoint; main: boolean }>()
const emit = defineEmits<{ resume: []; openSaves: [] }>()

const { t } = useI18n()

const from = computed(() => resumeSource(props.point))
</script>

<template>
  <Card :pad="CardPad.Tight" class="min-w-0 flex-row gap-10">
    <!-- 96px is the picture's own width, kept 3:2 at the top of the card (`self-start`: it never
         stretches to the card's height); a game that is not reachable fades it to 45%
         (Cowork: `disabled` is 40%). -->
    <SaveThumb
      :src="point?.thumbUrl"
      crop
      :class="
        cn(
          'aspect-3/2 w-continue-thumb flex-none self-start rounded-control',
          !game.available && 'opacity-missing',
        )
      "
      :style="{ background: game.colour }"
    />
    <div class="flex min-w-0 flex-1 flex-col gap-4">
      <div class="line-clamp-2 font-semibold wrap-anywhere">{{ game.title }}</div>
      <div class="text-caption text-ink-muted tabular-nums">
        {{
          t('library.continue.meta', {
            console: t(`consoleShort.${game.console}`),
            time: playTimeLabel(game.playTimeSecs),
            when: lastPlayedLabel(game.lastPlayed ?? 0),
          })
        }}
      </div>
      <div class="mt-auto flex flex-wrap gap-6">
        <Button
          :variant="main ? ButtonVariant.Primary : ButtonVariant.Soft"
          :size="ButtonSize.Compact"
          :data-nav-initial="main ? '' : undefined"
          :disabled="!game.available"
          :icon="Play"
          icon-filled
          @click="emit('resume')"
        >
          {{ t(point?.none ? 'library.hero.play' : 'library.hero.resume') }}
        </Button>
        <Button :size="ButtonSize.Compact" :icon="Save" @click="emit('openSaves')">
          {{ t('shell.saves') }}
        </Button>
      </div>
      <div v-if="from" class="text-fine text-ink-faint">
        {{ t(from.key, from.params ?? {}) }}
      </div>
    </div>
  </Card>
</template>
