<script setup lang="ts">
// L12 «Scegli la copertina» (board L12, Stati; Trello #85): one dialog, three steps.
// 1. The choice (CoverChoices): generated, the ROM icon, «Da uno screenshot», «Immagine tua».
//    Generated and icon apply at once and close.
// 2. «Scegli uno screenshot» (ScreenshotGrid): this game's screenshots.
// 3. «Adatta la copertina» (CoverCrop): the screenshot, or the image the native picker gave
//    (cancelling the picker keeps the choice open), framed in the 2:3 cover; Rust makes the
//    600 × 900 cover.
// On the Dialog primitive (wide, over the blurred screen; focus trap, `aria-modal`); each step has
// its own nav scope inside the dialog's, its first control focused. B goes back a step (and
// cancels from the choice); Esc does the same; a click outside does nothing, as before.
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Dialog, DialogContent, DialogSize } from '@/components/ui/dialog'
import { emu, type GameScreenshot, type LibraryGame } from '@/emu'
import { errorText } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import CoverChoices from './CoverChoices.vue'
import CoverCrop from './CoverCrop.vue'
import ScreenshotGrid from './ScreenshotGrid.vue'
import { choiceNext, type CoverChoice } from './coverChoice'
import type { CropSource } from './useCoverCrop'

const props = defineProps<{ game: LibraryGame }>()
const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()

type Step = 'choose' | 'shots' | 'crop'
const step = ref<Step>('choose')
const choice = ref<CoverChoice>(props.game.cover)
const shots = ref<GameScreenshot[] | null>(null)
const shot = ref<string | null>(null)
const source = ref<CropSource | null>(null)
const busy = ref(false)
const error = ref<string | null>(null)
const requests = latest()

onMounted(async () => {
  const id = props.game.id
  try {
    const list = await emu.library.screenshots(id)
    if (props.game.id === id) shots.value = list
  } catch (e) {
    if (props.game.id !== id) return
    shots.value = []
    error.value = errorText(e)
  }
})

async function next() {
  if (busy.value) return
  error.value = null
  switch (choiceNext(choice.value)) {
    case 'screenshots':
      step.value = 'shots'
      return
    case 'pick':
      return pick()
    case 'apply':
      return apply()
  }
}

/** Generated or icon: at once. */
async function apply() {
  const fresh = requests.next()
  busy.value = true
  try {
    await emu.library.setCover(props.game.id, choice.value === 'icon' ? 'icon' : 'generated')
    if (fresh()) emit('close')
  } catch (e) {
    if (fresh()) error.value = errorText(e)
  } finally {
    if (fresh()) busy.value = false
  }
}

/** «Immagine tua»: the native picker, then «Adatta la copertina». */
async function pick() {
  const fresh = requests.next()
  busy.value = true
  try {
    const picked = await emu.library.pickCoverImage()
    if (!fresh() || !picked) return
    source.value = {
      source: { kind: 'picked', token: picked.token },
      url: picked.url,
      width: picked.width,
      height: picked.height,
    }
    step.value = 'crop'
  } catch (e) {
    if (fresh()) error.value = errorText(e)
  } finally {
    if (fresh()) busy.value = false
  }
}

function frameShot(s: GameScreenshot) {
  shot.value = s.name
  source.value = {
    source: { kind: 'screenshot', name: s.name },
    url: s.url,
    width: s.width,
    height: s.height,
  }
  step.value = 'crop'
}

/** B / Esc / «Indietro»: one step back, or closed from the choice. */
function back() {
  if (step.value === 'crop') {
    step.value = source.value?.source.kind === 'screenshot' ? 'shots' : 'choose'
  } else if (step.value === 'shots') {
    step.value = 'choose'
  } else {
    emit('close')
  }
}

const keepOpen = (e: Event) => e.preventDefault()
function onEscape(e: KeyboardEvent) {
  e.preventDefault()
  back()
}
</script>

<template>
  <Dialog :open="true" @update:open="open => open || emit('close')">
    <DialogContent
      :size="DialogSize.Wide"
      :aria-label="t('library.cover.title', { title: game.title })"
      :aria-describedby="undefined"
      @interact-outside="keepOpen"
      @escape-key-down="onEscape"
    >
      <CoverChoices
        v-if="step === 'choose'"
        v-model="choice"
        :game="game"
        :shots="shots"
        :busy="busy"
        :error="error"
        @cancel="emit('close')"
        @next="next"
      />
      <ScreenshotGrid
        v-else-if="step === 'shots' && shots"
        :shots="shots"
        :chosen="shot"
        @back="back"
        @next="frameShot"
      />
      <CoverCrop
        v-else-if="step === 'crop' && source"
        :game="game.id"
        :source="source"
        @back="back"
        @done="emit('close')"
      />
    </DialogContent>
  </Dialog>
</template>
