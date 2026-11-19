<script setup lang="ts">
// One game's cover (board L01): the user's image, or the ROM icon on the game's colour with the
// title, or the generated cover (colour + title). The console badge is the short word (GBA, DS),
// the only place the Glossary allows it. A user image that fails to load falls back to generated.
//
// The screens that place a cover size it with a class of their own: a utility merges with the
// cover's own (cn), a scoped rule of a legacy screen (GameDetails, SavesView, FirstRun) is
// unlayered and wins over them.
import { computed, ref, watch, type HTMLAttributes } from 'vue'
import { useI18n } from 'vue-i18n'
import { cn } from '@/lib/cn'
import type { LibraryGame } from '../emu'

const props = withDefaults(
  defineProps<{ game: LibraryGame; badge?: boolean; class?: HTMLAttributes['class'] }>(),
  { badge: true, class: undefined },
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
  <!-- The title's 19px line and the badge's 16px line and 0.04em tracking are off the type
       scale (Cowork, spec Decision 5). -->
  <div
    :class="
      cn(
        'relative flex aspect-[2/3] items-end overflow-hidden rounded-10 p-12 text-left text-heading leading-[19px] text-on-cover',
        props.class,
      )
    "
    :style="{ background: game.colour }"
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
      class="pixelated absolute top-[38%] left-1/2 w-[40%] -translate-1/2 rounded-6"
      :src="image"
      alt=""
      @error="broken = true"
    />
    <span
      v-if="badge"
      class="absolute top-8 left-8 rounded-4 bg-cover-badge px-6 py-px text-overline leading-[16px] tracking-[0.04em]"
      >{{ t(`consoleShort.${game.console}`) }}</span
    >
    <span v-if="!userImage" class="relative wrap-anywhere">{{ game.title }}</span>
  </div>
</template>
