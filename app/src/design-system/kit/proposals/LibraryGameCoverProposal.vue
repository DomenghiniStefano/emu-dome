<script setup lang="ts">
// Kit proposal (library group, «Game cover»): GameCover with the «missing» look drawn in grey with
// the dashed edge instead of faded (the faded cover washes out on the light theme). Only shown on
// the Kit page; GameCover itself is unchanged until the owner decides.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CoverGame } from '@/components/cover/GameCover.vue'
import {
  CoverSize,
  coverBadgeVariants,
  coverTitleVariants,
  coverVariants,
} from '@/components/cover/variants'
import { cn } from '@/design-system/lib/cn'

const props = withDefaults(
  defineProps<{
    game: CoverGame
    size?: typeof CoverSize.Fill | typeof CoverSize.Small
    missing?: boolean
  }>(),
  { size: CoverSize.Fill, missing: false },
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
        missing && 'outline outline-offset-inset outline-line-strong outline-dashed grayscale',
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
      class="pixelated absolute top-[38%] left-1/2 w-2/5 -translate-1/2 rounded-control"
      :src="image"
      alt=""
    />
    <span :class="coverBadgeVariants({ size })">{{ t(`consoleShort.${game.console}`) }}</span>
    <span v-if="!userImage" :class="coverTitleVariants({ size })">{{ game.title }}</span>
  </div>
</template>
