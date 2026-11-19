<script setup lang="ts">
// One game of «Continua a giocare»: the newest quick save's picture (or the game's colour), the
// title and when it was played, «Riprendi» (or «Gioca» with nothing saved) and «Salvataggi»,
// and where «Riprendi» picks up. The first card's «Riprendi» is the view's main action.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import type { LibraryGame } from '../../emu'
import type { ResumePoint } from '../../gameDetails'
import { lastPlayedLabel, playTimeLabel } from '../../i18n/labels'
import { resumeSource } from './libraryView'

const props = defineProps<{ game: LibraryGame; point?: ResumePoint; main: boolean }>()
const emit = defineEmits<{ resume: []; openSaves: [] }>()

const { t } = useI18n()

const from = computed(() => resumeSource(props.point))
</script>

<template>
  <div class="flex min-w-0 gap-12 rounded-10 border border-line bg-surface p-10">
    <!-- 96px is the picture's own width; a game that is not reachable fades it to 45%
         (Cowork: `disabled` is 40%). -->
    <span
      class="aspect-3/2 w-continue-thumb flex-none overflow-hidden rounded-6"
      :class="{ 'opacity-missing': !game.available }"
      aria-hidden="true"
      :style="{ background: game.colour }"
    >
      <img
        v-if="point?.thumbUrl"
        class="pixelated block size-full object-cover"
        :src="point.thumbUrl"
        alt=""
      />
    </span>
    <div class="flex min-w-0 flex-1 flex-col gap-4">
      <div class="truncate font-semibold">{{ game.title }}</div>
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
          @click="emit('resume')"
        >
          {{ t(point?.none ? 'library.hero.play' : 'library.hero.resume') }}
        </Button>
        <Button :size="ButtonSize.Compact" @click="emit('openSaves')">
          {{ t('shell.saves') }}
        </Button>
      </div>
      <div v-if="from" class="text-fine text-ink-faint">
        {{ t(from.key, from.params ?? {}) }}
      </div>
    </div>
  </div>
</template>
