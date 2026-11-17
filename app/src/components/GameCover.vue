<script setup lang="ts">
// One game's cover (board L01): the user's image, or the ROM icon on the game's colour with the
// title, or the generated cover (colour + title). The console badge is the short word (GBA, DS),
// the only place the Glossary allows it. A user image that fails to load falls back to generated.
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { LibraryGame } from '../emu'

const props = withDefaults(defineProps<{ game: LibraryGame; badge?: boolean }>(), {
  badge: true,
})

const { t } = useI18n()

const broken = ref(false)
watch(
  () => props.game.coverUrl,
  () => (broken.value = false),
)

const image = computed(() =>
  !broken.value && props.game.cover !== 'generated' ? props.game.coverUrl : null,
)
</script>

<template>
  <div class="cover" :style="{ background: game.colour }">
    <img
      v-if="image && game.cover === 'user'"
      class="art"
      :src="image"
      alt=""
      @error="broken = true"
    />
    <img v-else-if="image" class="icon" :src="image" alt="" @error="broken = true" />
    <span v-if="badge" class="badge">{{ t(`consoleShort.${game.console}`) }}</span>
    <span v-if="!(image && game.cover === 'user')" class="title">{{ game.title }}</span>
  </div>
</template>

<style scoped>
.cover {
  position: relative;
  aspect-ratio: 2 / 3;
  border-radius: var(--radius-10);
  overflow: hidden;
  display: flex;
  align-items: flex-end;
  padding: 12px;
  color: #fff;
  font-weight: 600;
  font-size: 15px;
  line-height: 19px;
  text-align: left;
}
.art {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}
/* The ROM icon is small pixel art (32×32 on NDS): scaled up crisp, in the upper middle. */
.icon {
  position: absolute;
  left: 50%;
  top: 38%;
  width: 40%;
  transform: translate(-50%, -50%);
  image-rendering: pixelated;
  border-radius: var(--radius-6);
}
.badge {
  position: absolute;
  top: 8px;
  left: 8px;
  padding: 1px 6px;
  border-radius: 4px;
  background: #000000a6;
  font-size: 11px;
  line-height: 16px;
  letter-spacing: 0.04em;
}
.title {
  position: relative;
  overflow-wrap: anywhere;
}
</style>
