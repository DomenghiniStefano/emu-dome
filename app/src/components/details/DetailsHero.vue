<script setup lang="ts">
// L04, the head of the game details (board «Dettagli»): «‹ Libreria», the cover, console, title,
// play time and last game, «Riprendi» (or «Gioca»), the favourite and «…». The cover's colour
// fades down behind it and glows around the cover.
// Phone in portrait («Tel-Dettagli»): ‹ and «…» on a bar, the cover beside the title, «Riprendi»
// full width under them.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft, Heart } from '@lucide/vue'
import type { LibraryGame } from '../../emu'
import type { ResumePoint } from '../../gameDetails'
import { playTimeLabel, saveWhenLabel } from '../../i18n/labels'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import GameCover from '../GameCover.vue'
import GameMoreButton from './GameMoreButton.vue'
import ResumeButton from './ResumeButton.vue'

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
    class="relative flex flex-col bg-cover-fade"
    :class="phone ? 'gap-16 px-16 pt-8' : 'gap-20 px-32 pt-24 pb-[28px]'"
    :style="{ '--cover-tint': game.colour }"
    :aria-label="game.title"
  >
    <div v-if="phone" class="-mx-8 flex items-center justify-between">
      <Button
        :variant="ButtonVariant.Plain"
        :size="ButtonSize.IconTouch"
        :aria-label="t('details.back')"
        :title="t('details.back')"
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

    <div class="flex items-end" :class="phone ? 'flex-nowrap gap-14' : 'flex-wrap gap-[28px]'">
      <GameCover
        class="flex-none"
        :class="phone ? 'w-[96px] shadow-cover-glow' : 'w-[168px] shadow-cover-hero'"
        :game="game"
        :badge="false"
      />
      <div
        class="flex min-w-0 flex-col"
        :class="phone ? 'flex-1 gap-4' : 'flex-[1_1_420px] gap-10'"
      >
        <div
          class="text-ink-muted uppercase"
          :class="phone ? 'text-caption font-semibold tracking-[0.08em]' : 'text-overline'"
        >
          {{ t(`console.${game.console}`) }}
        </div>
        <h1
          class="m-0 wrap-anywhere"
          :class="phone ? 'text-[22px]/[28px] font-semibold tracking-[-0.02em]' : 'text-display'"
        >
          {{ game.title }}
        </h1>
        <div class="text-ink-muted tabular-nums" :class="phone ? 'text-body' : 'text-lead'">
          {{ meta }}
        </div>
        <div v-if="!phone" class="mt-6 flex flex-wrap gap-8">
          <ResumeButton
            :phone="false"
            :available="game.available"
            :fresh="!!resume?.none"
            @click="emit('play')"
          />
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
      phone
      :available="game.available"
      :fresh="!!resume?.none"
      @click="emit('play')"
    />
  </section>
</template>
