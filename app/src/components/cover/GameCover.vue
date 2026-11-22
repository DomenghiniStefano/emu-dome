<script setup lang="ts">
// One game's cover (board L01): the user's image, or the ROM icon on the game's colour with the
// title, or the generated cover (colour + title). The console badge is the short word (GBA, DS),
// the only place the Glossary allows it. A user image that fails to load falls back to generated.
//
// Its size is one of CoverSize (cover/variants.ts); a screen adds only placement (flex-none, a
// shadow, a state), never a size of its own.
//
// The images load lazily and decode off the main thread: a library of hundreds of covers fetches
// only those near the screen.
//
// `missing`: the game file is not on this device (a game only on Google Drive, #31): faded to
// `opacity-missing` and edged in dashes, the look of a game whose folder is not reachable.
import { computed, ref, watch, type HTMLAttributes } from 'vue'
import { useI18n } from 'vue-i18n'
import { cn } from '@/lib/cn'
import type { LibraryGame } from '@/emu'
import {
  CoverSize,
  coverBadgeVariants,
  coverIconVariants,
  coverTitleVariants,
  coverVariants,
  coverVersionsVariants,
} from './variants'

/** What a cover shows of a game: a library game, or one known only by its title (Drive). */
export type CoverGame = Pick<LibraryGame, 'console' | 'title' | 'colour' | 'cover' | 'coverUrl'>

const props = withDefaults(
  defineProps<{
    game: CoverGame
    size?: CoverSize
    /** The console's short name in a corner. */
    badge?: boolean
    /** The game's title over a cover without a picture (off on the thumbnails). */
    title?: boolean
    missing?: boolean
    /** How many regional versions its family holds (#102): the «N versioni» tag from 2. */
    versions?: number
    class?: HTMLAttributes['class']
  }>(),
  { size: CoverSize.Fill, badge: true, title: true, missing: false, versions: 0, class: undefined },
)

const { t } = useI18n()

const broken = ref(false)
watch(
  () => props.game.coverUrl,
  () => (broken.value = false),
)

const image = computed(() =>
  !broken.value && props.game.cover !== 'generated' ? props.game.coverUrl : null,
)
const userImage = computed(() => image.value !== null && props.game.cover === 'user')
</script>

<template>
  <div
    :class="
      cn(
        coverVariants({ size }),
        'bg-cover-tint',
        missing &&
          'opacity-missing outline outline-offset-inset outline-line-strong outline-dashed',
        props.class,
      )
    "
    :style="{ '--cover-tint': game.colour }"
  >
    <img
      v-if="userImage"
      class="absolute inset-0 size-full object-cover"
      :src="image ?? ''"
      alt=""
      loading="lazy"
      decoding="async"
      @error="broken = true"
    />
    <!-- The ROM icon is small pixel art (32×32 on NDS): scaled up crisp (coverIconVariants). -->
    <img
      v-else-if="image"
      :class="coverIconVariants({ size })"
      :src="image"
      alt=""
      loading="lazy"
      decoding="async"
      @error="broken = true"
    />
    <span v-if="badge" :class="coverBadgeVariants({ size })">{{
      t(`consoleShort.${game.console}`)
    }}</span>
    <span v-if="versions > 1" :class="coverVersionsVariants({ size })">{{
      t('library.versions.tag', { n: versions }, versions)
    }}</span>
    <span v-if="title && !userImage" :class="coverTitleVariants({ size })">{{ game.title }}</span>
  </div>
</template>
