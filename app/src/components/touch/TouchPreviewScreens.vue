<script setup lang="ts">
// C06 with no game (Settings, Trello #111): the console's screens drawn where the game would
// show them, as the C09 editor's placeholders (a raised surface with the screen's name), under
// the controls. Only a picture: a press goes through to the editor.
import { useI18n } from 'vue-i18n'
import type { ConsoleId, ScreenPlacement } from '@/emu'

const props = defineProps<{
  console: ConsoleId
  placements: ScreenPlacement[]
}>()

const { t } = useI18n()
const px = (v: number) => `${v}px`
const box = (p: ScreenPlacement) => ({
  left: px(p.dst.x),
  top: px(p.dst.y),
  width: px(p.dst.w),
  height: px(p.dst.h),
})
/** A one-screen console's picture is named by the console, a DS / 3DS screen by its place. */
const name = (p: ScreenPlacement) =>
  p.id === 'main' ? t(`console.${props.console}`) : t(`screenEditor.short.${p.id}`)
</script>

<template>
  <div class="pointer-events-none absolute inset-0" aria-hidden="true">
    <span
      v-for="p in placements"
      :key="p.id"
      class="absolute flex items-center justify-center bg-surface-raised font-semibold text-ink-muted outline outline-line-strong"
      :style="box(p)"
    >
      {{ name(p) }}
    </span>
  </div>
</template>
