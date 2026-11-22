<script setup lang="ts">
// Kit proposal (library group, «Continue card»): ContinueCard with its picture kept at 3:2 at the
// top of the card (`self-start`: in the card's row it no longer stretches to the card's height)
// and the title on up to two lines instead of cut after a few letters. Only shown on the Kit page.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardPad } from '@/components/ui/card'
import { cn } from '@/lib/cn'
import SaveThumb from '@/components/saves/SaveThumb.vue'
import type { LibraryGame } from '@/emu'
import type { ResumePoint } from '@/lib/gameDetails'
import { lastPlayedLabel, playTimeLabel } from '@/i18n/labels'
import { resumeSource } from '@/components/library/libraryView'

const props = defineProps<{ game: LibraryGame; point?: ResumePoint; main: boolean }>()

const { t } = useI18n()

const from = computed(() => resumeSource(props.point))
</script>

<template>
  <Card :pad="CardPad.Tight" class="min-w-0 flex-row gap-12">
    <SaveThumb
      :src="point?.thumbUrl"
      crop
      :class="
        cn(
          'aspect-3/2 w-continue-thumb flex-none self-start rounded-6',
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
          :aria-disabled="!game.available || undefined"
        >
          {{ t(point?.none ? 'library.hero.play' : 'library.hero.resume') }}
        </Button>
        <Button :size="ButtonSize.Compact">{{ t('shell.saves') }}</Button>
      </div>
      <div v-if="from" class="text-fine text-ink-faint">
        {{ t(from.key, from.params ?? {}) }}
      </div>
    </div>
  </Card>
</template>
