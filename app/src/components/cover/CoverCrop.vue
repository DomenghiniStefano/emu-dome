<script setup lang="ts">
// «Adatta la copertina» (Trello #85, the owner's mockup 2026-11-21, section 3): the image in the
// library's 2:3 frame as the cover will be, by default whole with the spare space filled by the
// same image enlarged, blurred and dimmed; «Intera / Riempi», the zoom, and the image moved by
// dragging it. No title on the cover. One step for a screenshot and for the user's own image,
// inside the dialog that opened it (CoverPicker, or CoverCropDialog from the game's screenshot
// toast). The state and the keys are useCoverCrop's, the geometry coverCrop.ts's; Rust makes the
// 600 × 900 cover from the view.
// Controller: its own nav scope inside the dialog's: the d-pad moves the image, L1 / R1 zoom, X
// switches the mode, A uses the cover (on a button, presses it), B goes back.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { DialogTitle, DialogTitleSize } from '@/components/ui/dialog'
import { Segmented, SegmentedItem } from '@/components/ui/segmented'
import { Slider } from '@/components/ui/slider'
import type { GameId } from '@/emu'
import { useNavScope } from '@/nav'
import CropHints from './CropHints.vue'
import { CropMode, ZOOM_MAX, ZOOM_MIN, ZOOM_STEP, zoomPercent } from './coverCrop'
import { useCoverCrop, type CropSource } from './useCoverCrop'

const props = defineProps<{ game: GameId; source: CropSource }>()
const emit = defineEmits<{ back: []; done: [] }>()

const { t } = useI18n()
const root = ref<HTMLElement | null>(null)
const frame = ref<HTMLElement | null>(null)
const crop = useCoverCrop(
  { game: props.game, source: props.source, done: () => emit('done') },
  frame,
)
const { place, mode, zoom, busy, error } = crop
useNavScope(root, { back: () => emit('back'), own: crop.own, autofocus: true })

const modeModel = computed({
  get: () => mode.value,
  set: (v: string) => crop.setMode(v === CropMode.Fill ? CropMode.Fill : CropMode.Whole),
})
const zoomModel = computed({
  get: () => zoom.value,
  set: (v: number) => crop.setZoom(v),
})
const zoomText = computed(() =>
  t('library.cover.crop.zoomValue', { value: zoomPercent(zoom.value) }),
)
/** Where the image sits in the frame (coverCrop.ts `placement`). */
const imageStyle = computed(() =>
  place.value
    ? {
        left: `${place.value.left}%`,
        top: `${place.value.top}%`,
        width: `${place.value.width}%`,
        height: `${place.value.height}%`,
      }
    : undefined,
)
</script>

<template>
  <div ref="root" class="flex flex-col gap-12">
    <DialogTitle as="div" :size="DialogTitleSize.Heading">
      {{ t('library.cover.crop.title') }}
    </DialogTitle>
    <div class="flex flex-wrap items-start gap-16">
      <!-- The cover as it will be: the backdrop (the image filling the frame, blurred, under the
           game scrim), then the image where the view puts it. -->
      <div
        ref="frame"
        tabindex="0"
        role="img"
        :aria-label="t('library.cover.crop.frame')"
        data-nav-initial
        class="relative aspect-2/3 w-cover-hero shrink-0 cursor-grab touch-none overflow-hidden rounded-card bg-letterbox select-none active:cursor-grabbing"
        @pointerdown="crop.pointerDown"
        @pointermove="crop.pointerMove"
        @pointerup="crop.pointerUp"
        @pointercancel="crop.pointerUp"
        @wheel="crop.wheel"
      >
        <img
          class="pointer-events-none absolute inset-0 size-full object-cover blur-dim"
          :src="source.url"
          alt=""
          draggable="false"
        />
        <span class="pointer-events-none absolute inset-0 bg-game-scrim" aria-hidden="true"></span>
        <img
          v-show="place"
          class="pointer-events-none absolute max-w-none"
          :style="imageStyle"
          :src="source.url"
          alt=""
          draggable="false"
          @load="crop.loaded"
        />
      </div>
      <div class="flex min-w-0 flex-1 flex-col gap-12">
        <Segmented
          v-model="modeModel"
          class="self-start"
          :aria-label="t('library.cover.crop.mode')"
        >
          <SegmentedItem :value="CropMode.Whole">{{ t('library.cover.crop.whole') }}</SegmentedItem>
          <SegmentedItem :value="CropMode.Fill">{{ t('library.cover.crop.fill') }}</SegmentedItem>
        </Segmented>
        <div class="flex items-center gap-8">
          <span class="text-caption text-ink-muted">{{ t('library.cover.crop.zoom') }}</span>
          <Slider
            v-model="zoomModel"
            :min="ZOOM_MIN"
            :max="ZOOM_MAX"
            :step="ZOOM_STEP"
            :label="t('library.cover.crop.zoom')"
            :value-text="zoomText"
            class="w-auto flex-1"
          />
          <span class="text-caption text-ink-muted tabular-nums">{{ zoomText }}</span>
        </div>
        <p class="m-0 text-caption text-ink-muted">{{ t('library.cover.crop.drag') }}</p>
        <CropHints />
      </div>
    </div>
    <p v-if="error" class="m-0 text-danger" role="alert">{{ error }}</p>
    <div class="flex justify-between gap-8">
      <Button :size="ButtonSize.Medium" @click="emit('back')">{{ t('library.cover.back') }}</Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        :disabled="busy || !place"
        @click="crop.use"
      >
        {{ t('library.cover.use') }}
      </Button>
    </div>
  </div>
</template>
