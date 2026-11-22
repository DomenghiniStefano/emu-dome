<script setup lang="ts">
// L04, the head of the game details (board «Dettagli»): «‹ Libreria», the cover, console, title,
// play time and last game, «Riprendi» (or «Gioca»), the favourite and «…». The cover's colour
// fades down behind it and glows around the cover.
// Phone in portrait («Tel-Dettagli»): ‹ and «…» on a bar, the cover beside the title, «Riprendi»
// full width under them.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft, Heart } from '@lucide/vue'
import type { LibraryGame } from '@/emu'
import type { ResumePoint } from '@/lib/gameDetails'
import { playTimeLabel, saveWhenLabel } from '@/i18n/labels'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import GameMoreButton from './GameMoreButton.vue'
import ResumeButton from './ResumeButton.vue'
import { Heading, HeadingVariant } from '@/components/ui/heading'

const props = defineProps<{ game: LibraryGame; phone: boolean; resume: ResumePoint | null }>()
const emit = defineEmits<{
  back: []
  play: []
  /** «Scegli la copertina…»: L12. */
  cover: []
  flags: [{ favourite?: boolean; hidden?: boolean }]
}>()

const { t } = useI18n()

const meta = computed(() => {
  const g = props.game
  if (g.lastPlayed === null) return t('library.neverPlayed')
  return t(props.phone ? 'details.metaShort' : 'details.meta', {
    time: playTimeLabel(g.playTimeSecs, true),
    when: saveWhenLabel(g.lastPlayed),
  })
})

const more = ref<InstanceType<typeof GameMoreButton> | null>(null)
defineExpose({
  /** The focus back on «…» (the cover picker closed). */
  focusMore: () => more.value?.focus(),
})
</script>

<template>
  <section
    class="relative flex flex-col gap-section-gap bg-cover-fade px-section-x pt-section-top pb-20 touch:gap-section-touch touch:px-section-touch touch:pt-8 touch:pb-0"
    :style="{ '--cover-tint': game.colour }"
    :aria-label="game.title"
  >
    <div v-if="phone" class="-mx-8 flex items-center justify-between">
      <Button
        :variant="ButtonVariant.Plain"
        :size="ButtonSize.IconTouch"
        :label="t('details.back')"
        @click="emit('back')"
      >
        <ChevronLeft :size="24" :stroke-width="1.75" aria-hidden="true" />
      </Button>
      <GameMoreButton
        ref="more"
        :game="game"
        phone
        :items="['cover', 'favourite', 'hide']"
        @cover="emit('cover')"
        @flags="flags => emit('flags', flags)"
      />
    </div>
    <Button
      v-else
      :variant="ButtonVariant.Text"
      :size="ButtonSize.Bare"
      class="self-start"
      @click="emit('back')"
    >
      <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
      {{ t('shell.library') }}
    </Button>

    <div class="flex flex-wrap items-end gap-20 touch:flex-nowrap touch:gap-14">
      <GameCover
        :size="CoverSize.Hero"
        class="flex-none shadow-cover-hero touch:shadow-cover-glow"
        :game="game"
        :badge="false"
      />
      <div class="flex min-w-0 grow basis-hero-text flex-col gap-8 touch:flex-1 touch:gap-4">
        <Heading as="div" :variant="HeadingVariant.Overline">
          {{ t(`console.${game.console}`) }}
        </Heading>
        <Heading as="h1" :variant="HeadingVariant.Hero">
          {{ game.title }}
        </Heading>
        <div class="text-lead text-ink-muted tabular-nums touch:text-body">
          {{ meta }}
        </div>
        <div v-if="!phone" class="mt-4 flex flex-wrap gap-8">
          <ResumeButton :available="game.available" :fresh="!!resume?.none" @click="emit('play')" />
          <Button :size="ButtonSize.Tall" @click="emit('flags', { favourite: !game.favourite })">
            <Heart
              :size="16"
              :stroke-width="1.75"
              :fill="game.favourite ? 'currentColor' : 'none'"
              aria-hidden="true"
            />
            {{ t(game.favourite ? 'library.menu.unfavourite' : 'library.menu.favourite') }}
          </Button>
          <GameMoreButton
            ref="more"
            :game="game"
            :phone="false"
            :items="['cover', 'hide']"
            @cover="emit('cover')"
            @flags="flags => emit('flags', flags)"
          />
        </div>
        <p v-if="!game.available" class="m-0 text-caption text-warning">
          {{ t('errors.libraryGameUnavailable') }}
        </p>
      </div>
    </div>
    <ResumeButton
      v-if="phone"
      :available="game.available"
      :fresh="!!resume?.none"
      @click="emit('play')"
    />
  </section>
</template>
