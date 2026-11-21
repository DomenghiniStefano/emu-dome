<script setup lang="ts">
// One game of the grid (#59 item 2): a click on the cover opens its details (L04), as does a
// right click; with the pointer over it or the focus in it, a ring, «Gioca» (starts the game)
// and «…» (CoverMenu: favourite, hide). A game that is not reachable (E01) is shown faded and
// not launchable.
//
// The cover's click area stops above the action strip, so the controller reaches «Gioca» and
// «…» by moving down from it (nav.ts moves by rects). The ring is the frame's, so the buttons
// inside draw none of their own (focus-ring-none / focus-ring-tight, utilities.css).
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Ellipsis, Play } from '@lucide/vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import type { LibraryGame } from '@/emu'
import { lastPlayedLabel, playTimeLabel } from '@/i18n/labels'
import CoverMenu from '@/components/cover/CoverMenu.vue'
import GameCover from '@/components/cover/GameCover.vue'
import { COVER_OF } from './useLibraryFocus'

const props = defineProps<{ game: LibraryGame; menuOpen: boolean }>()
const emit = defineEmits<{
  details: []
  play: []
  toggleMenu: [Event]
  closeMenu: []
  flags: [{ favourite?: boolean; hidden?: boolean }]
}>()

const { t } = useI18n()

const more = ref<InstanceType<typeof Button> | null>(null)
const moreEl = computed(() => (more.value?.$el as HTMLElement | undefined) ?? null)

const off = computed(() => !props.game.available)
const meta = computed(() =>
  props.game.lastPlayed === null
    ? t('library.neverPlayed')
    : t('library.meta', {
        time: playTimeLabel(props.game.playTimeSecs),
        when: lastPlayedLabel(props.game.lastPlayed),
      }),
)
</script>

<template>
  <div class="flex min-w-0 flex-col gap-8" :data-game="game.id">
    <div
      class="group/frame relative rounded-10 shadow-card outline-ring outline-offset-ring outline-transparent transition-[outline-color] hover:outline-focus-ring focus-within:outline-focus-ring data-open:outline-focus-ring"
      :data-open="menuOpen ? '' : undefined"
      @contextmenu.prevent="emit('details')"
    >
      <!-- Faded to 45% (Cowork: `disabled` is 40%), edged in dashes. -->
      <GameCover
        :game="game"
        :class="{
          'opacity-missing outline outline-offset-inset outline-line-strong outline-dashed': off,
        }"
      />
      <!-- The action strip's height (52), which the click area stops above. -->
      <Button
        :variant="ButtonVariant.Bare"
        :size="ButtonSize.Area"
        class="focus-ring-none absolute inset-x-0 top-0 bottom-52 rounded-t-10 rounded-b-none"
        v-bind="{ [COVER_OF]: game.id }"
        :aria-label="t('library.openDetails', { title: game.title })"
        @click="emit('details')"
      ></Button>
      <div
        class="absolute right-8 bottom-8 left-8 flex gap-6 opacity-0 transition-opacity group-hover/frame:opacity-100 group-focus-within/frame:opacity-100 group-data-open/frame:opacity-100"
      >
        <Button
          :variant="ButtonVariant.Inverse"
          :size="ButtonSize.CoverAction"
          :fade="ButtonFade.None"
          class="focus-ring-tight min-w-0 flex-1"
          :aria-disabled="off || undefined"
          @click="emit('play')"
        >
          <Play :size="14" fill="currentColor" :stroke-width="0" aria-hidden="true" />
          {{ t('library.hero.play') }}
        </Button>
        <div class="relative">
          <Button
            ref="more"
            :variant="ButtonVariant.Veil"
            :size="ButtonSize.CoverActionIcon"
            class="focus-ring-tight"
            :aria-label="t('library.moreFor', { title: game.title })"
            :title="t('library.more')"
            aria-haspopup="menu"
            :aria-expanded="menuOpen"
            @click="(e: Event) => emit('toggleMenu', e)"
          >
            <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
          </Button>
          <CoverMenu
            v-if="menuOpen"
            :game="game"
            :items="['favourite', 'hide']"
            :anchor="moreEl"
            @flags="f => emit('flags', f)"
            @close="emit('closeMenu')"
          />
        </div>
      </div>
    </div>
    <div class="flex flex-col gap-2" :class="{ 'opacity-missing-label': off }">
      <div class="min-w-0 font-medium wrap-anywhere">{{ game.title }}</div>
      <div class="text-caption text-ink-muted tabular-nums">{{ meta }}</div>
    </div>
  </div>
</template>
