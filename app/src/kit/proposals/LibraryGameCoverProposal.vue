<script setup lang="ts">
// Kit proposal (library group, «Game cover»): GameCover with its title held to a few lines, so a
// long title never climbs under the console badge, and the «missing» look drawn in grey with the
// dashed edge instead of faded (the faded cover washes out on the light theme). Only shown on the
// Kit page; GameCover itself is unchanged until the owner decides.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CoverGame } from '@/components/cover/GameCover.vue'
import { CoverSize, coverBadgeVariants, coverVariants } from '@/components/cover/variants'
import { cn } from '@/lib/cn'

const props = withDefaults(
  defineProps<{
    game: CoverGame
    size?: typeof CoverSize.Fill | typeof CoverSize.Small
    missing?: boolean
    /** Off: the current faded look, to compare only the title. */
    grey?: boolean
  }>(),
  { size: CoverSize.Fill, missing: false, grey: true },
)

const { t } = useI18n()

const image = computed(() => (props.game.cover !== 'generated' ? props.game.coverUrl : null))
const userImage = computed(() => image.value !== null && props.game.cover === 'user')
</script>

<template>
  <div
    :class="
      cn(
        coverVariants({ size }),
        'bg-cover-tint',
        missing && 'outline outline-offset-inset outline-line-strong outline-dashed',
        missing && (grey ? 'grayscale' : 'opacity-missing'),
      )
    "
    :style="{ '--cover-tint': game.colour }"
  >
    <img
      v-if="userImage"
      class="absolute inset-0 size-full object-cover"
      :src="image ?? ''"
      alt=""
    />
    <img
      v-else-if="image"
      class="pixelated absolute top-[38%] left-1/2 w-2/5 -translate-1/2 rounded-6"
      :src="image"
      alt=""
    />
    <span :class="coverBadgeVariants({ size })">{{ t(`consoleShort.${game.console}`) }}</span>
    <span
      v-if="!userImage"
      class="relative wrap-anywhere"
      :class="size === CoverSize.Small ? 'line-clamp-3' : 'line-clamp-4'"
      >{{ game.title }}</span
    >
  </div>
</template>
