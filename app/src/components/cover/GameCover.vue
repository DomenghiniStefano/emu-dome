<script setup lang="ts">
// One game's cover (board L01): the user's image, or the ROM icon on the game's colour with the
// title, or the generated cover (colour + title). The console badge is the short word (GBA, DS),
// the only place the Glossary allows it. A user image that fails to load falls back to generated.
//
// Its size is one of CoverSize (cover/variants.ts); a screen adds only placement (flex-none, a
// shadow, a state), never a size of its own.
import { computed, ref, watch, type HTMLAttributes } from 'vue'
import { useI18n } from 'vue-i18n'
import { cn } from '@/lib/cn'
import type { LibraryGame } from '@/emu'
import { CoverSize, coverVariants } from './variants'

const props = withDefaults(
  defineProps<{
    game: LibraryGame
    size?: CoverSize
    /** The console's short name in a corner. */
    badge?: boolean
    /** The game's title over a cover without a picture (off on the thumbnails). */
    title?: boolean
    class?: HTMLAttributes['class']
  }>(),
  { size: CoverSize.Fill, badge: true, title: true, class: undefined },
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
    :class="cn(coverVariants({ size }), 'bg-cover-tint', props.class)"
    :style="{ '--cover-tint': game.colour }"
  >
    <img
      v-if="userImage"
      class="absolute inset-0 size-full object-cover"
      :src="image ?? ''"
      alt=""
      @error="broken = true"
    />
    <!-- The ROM icon is small pixel art (32×32 on NDS): scaled up crisp, in the upper middle. -->
    <img
      v-else-if="image"
      class="pixelated absolute top-[38%] left-1/2 w-2/5 -translate-1/2 rounded-6"
      :src="image"
      alt=""
      @error="broken = true"
    />
    <span
      v-if="badge"
      class="absolute top-8 left-8 rounded-badge bg-cover-badge px-6 py-px text-console-badge"
      >{{ t(`consoleShort.${game.console}`) }}</span
    >
    <span v-if="title && !userImage" class="relative wrap-anywhere">{{ game.title }}</span>
  </div>
</template>
