<script setup lang="ts">
// «Adatta la copertina» for a screenshot just taken in game (the screenshot toast's «Usa come
// copertina», Trello #85): the same step as in L12, alone in the wide dialog. App.vue shows it
// once the interface can (the quick menu open, or the game closed). The screenshot is found in
// the game's list (its address and size are the backend's); one that is gone says so.
// Controller: the step's own nav scope; B, Esc and «Indietro» close it.
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Dialog, DialogContent, DialogSize } from '@/design-system/components/dialog'
import { emu } from '@/emu'
import { useErrorSink } from '@/composables/useToasts'
import CoverCrop from './CoverCrop.vue'
import type { CoverRequest } from './useCoverRequest'
import type { CropSource } from './useCoverCrop'

const props = defineProps<{ request: CoverRequest }>()
const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()
const failWith = useErrorSink()
const source = ref<CropSource | null>(null)

onMounted(async () => {
  const { game, name } = props.request
  try {
    const shots = await emu.library.screenshots(game)
    // A newer request replaced this one meanwhile: its own dialog reads its own list.
    if (props.request.game !== game || props.request.name !== name) return
    const shot = shots.find(s => s.name === name)
    if (!shot) {
      failWith({ code: 'coverInvalid', params: { reason: 'missing' } })
      emit('close')
      return
    }
    source.value = {
      source: { kind: 'screenshot', name },
      url: shot.url,
      width: shot.width,
      height: shot.height,
    }
  } catch (e) {
    failWith(e)
    emit('close')
  }
})

const keepOpen = (e: Event) => e.preventDefault()
</script>

<template>
  <Dialog :open="true" @update:open="open => open || emit('close')">
    <DialogContent
      :size="DialogSize.Wide"
      :aria-label="t('library.cover.crop.title')"
      :aria-describedby="undefined"
      @interact-outside="keepOpen"
    >
      <CoverCrop
        v-if="source"
        :game="request.game"
        :source="source"
        @back="emit('close')"
        @done="emit('close')"
      />
    </DialogContent>
  </Dialog>
</template>
