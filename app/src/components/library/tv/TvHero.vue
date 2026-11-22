<script setup lang="ts">
// The focused game, large, over the shelves (TV mode, #30, mockup «TV-Libreria» option A): the
// overline («Continua a giocare» once played, else its console), the title, when and how long it
// was played, the main action («Riprendi» with something to resume, else «Gioca») and «Dettagli
// del gioco», each with the controller button that does it from a cover too; its cover with the
// cover's glow. It follows the focus across the shelves.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import TvGlyph from '@/components/shell/TvGlyph.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { KbdVariant } from '@/components/ui/kbd'
import type { LibraryGame } from '@/emu'
import { playTimeLabel, saveWhenLabel } from '@/i18n/labels'
import { heroOverline } from './tvShelves'

const props = defineProps<{
  game: LibraryGame
  /** Something to pick up from: «Riprendi», else «Gioca». */
  resumable: boolean
}>()
const emit = defineEmits<{ play: []; details: [] }>()

const { t } = useI18n()

const overline = computed(() => heroOverline(props.game))
const meta = computed(() => {
  const console = t(`console.${props.game.console}`)
  const when = props.game.lastPlayed
  return when === null
    ? t('tv.hero.metaNew', { console })
    : t('tv.hero.meta', {
        console,
        time: playTimeLabel(props.game.playTimeSecs, true),
        when: saveWhenLabel(when),
      })
})
</script>

<template>
  <!-- As tall as the mockup's band, less on a short screen (40% of it), so a shelf shows under it. -->
  <section
    class="flex h-[min(var(--spacing-tv-hero),40vh)] shrink-0 items-center gap-tv-hero-gap px-tv-gutter py-20"
    :aria-label="game.title"
  >
    <div class="flex min-w-0 flex-1 flex-col gap-16">
      <span class="text-tv-overline text-ink-muted uppercase">
        {{ t(overline.key, overline.params ?? {}) }}
      </span>
      <h1 class="m-0 truncate text-tv-display">{{ game.title }}</h1>
      <span class="truncate text-tv-lead text-ink-muted tabular-nums">{{ meta }}</span>
      <div class="mt-16 flex gap-16">
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.Tv"
          data-nav-initial
          :aria-disabled="!game.available || undefined"
          @click="emit('play')"
        >
          <TvGlyph action="confirm" :variant="KbdVariant.OnAccent" aria-hidden="true" />
          {{ t(resumable ? 'library.hero.resume' : 'library.hero.play') }}
        </Button>
        <Button :size="ButtonSize.Tv" @click="emit('details')">
          <TvGlyph action="more" aria-hidden="true" />
          {{ t('tv.hint.details') }}
        </Button>
      </div>
    </div>
    <GameCover
      :game="game"
      :size="CoverSize.TvHero"
      :badge="false"
      :title="false"
      class="shrink-0 shadow-cover-hero"
    />
  </section>
</template>
