<script setup lang="ts">
// One horizontal shelf of the TV library (#30, component «TvShelf»): its title and a row of
// covers running off the screen's right edge. The focused cover grows (168 -> 200) and lifts, the
// TV focus ring around it (styles.css); the row scrolls to keep it in view. A (a click) plays the
// game, X (a right click) opens its details; the focus tints the haze and fills the hero.
// It renders `limit` covers (TvLibrary grows it as the focus nears the end: tvShelves.ts).
// The steps between covers and shelves are TvLibrary's (`useNavGrid`): `data-shelf-row` /
// `data-shelf-index` say where a cover is.
import { useI18n } from 'vue-i18n'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize, CoverStackLayer, coverStackVariants } from '@/components/cover/variants'
import { COVER_OF } from '@/components/library/useLibraryFocus'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import type { LibraryGame } from '@/emu'
import { cn } from '@/lib/cn'
import { versionCount } from '@/lib/families'
import { libraryFamilies } from '@/lib/library'
import { shelfTitle, type Shelf } from './tvShelves'

const props = defineProps<{ shelf: Shelf; limit: number }>()

/** A family's tile (#102): its versions, from 2 a deck and the tag. «Continua a giocare» holds
 * the versions played, each its own. */
const versions = (g: LibraryGame) =>
  props.shelf.id === 'continue' ? 0 : versionCount(libraryFamilies.value, g.id)
const emit = defineEmits<{
  play: [LibraryGame]
  details: [LibraryGame]
  /** The cover at `index` took the focus. */
  focused: [LibraryGame, index: number]
}>()

const { t } = useI18n()
</script>

<template>
  <section class="flex flex-col gap-16" :aria-label="t(shelfTitle(shelf.id))">
    <h2 class="m-0 text-tv-heading">{{ t(shelfTitle(shelf.id)) }}</h2>
    <!-- Room around the covers for the focus ring (4 + 4), which the scrolling row would cut. -->
    <div
      class="-m-8 flex h-[calc(var(--spacing-tv-cover-focus)*3/2+var(--spacing-16))] items-end gap-tv-cover-gap overflow-x-auto p-8 [scrollbar-width:none]"
      data-shelf-row
    >
      <Button
        v-for="(g, i) in shelf.games.slice(0, limit)"
        :key="g.id"
        :variant="ButtonVariant.Bare"
        :size="ButtonSize.Area"
        :class="
          cn(
            'relative w-tv-cover flex-none scroll-mt-48 scroll-mb-16 rounded-card whitespace-normal transition-[width] duration-fast ease-out focus:w-tv-cover-focus focus:shadow-tv-lift',
            !g.available && 'opacity-missing',
          )
        "
        :data-shelf-index="i"
        v-bind="{ [COVER_OF]: g.id }"
        :aria-label="t('library.open', { title: g.title })"
        @click="emit('play', g)"
        @contextmenu.prevent="emit('details', g)"
        @focus="emit('focused', g, i)"
      >
        <template v-if="versions(g) > 1">
          <span :class="coverStackVariants({ layer: CoverStackLayer.Far })" aria-hidden="true" />
          <span :class="coverStackVariants({ layer: CoverStackLayer.Near })" aria-hidden="true" />
        </template>
        <GameCover :game="g" :size="CoverSize.Shelf" :versions="versions(g)" />
      </Button>
    </div>
  </section>
</template>
