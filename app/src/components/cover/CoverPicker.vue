<script setup lang="ts">
// «Scegli la copertina» (board L12, Stati): generated, the ROM icon (only when the ROM has one),
// or the user's image (the backend opens the image picker; cancelling it keeps this open).
// On the Dialog primitive (wide, over the blurred screen; focus trap, `aria-modal`) with the
// covers as a RadioGroup (the arrows along the row choose).
// Controller / keyboard: the dialog's own nav scope, the chosen cover focused first; B / Esc
// cancel; a click outside does nothing, as before.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Plus } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogSize,
  DialogTitle,
  DialogTitleSize,
} from '@/components/ui/dialog'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { emu, type CoverKind, type LibraryGame } from '@/emu'
import { errorText } from '@/i18n/labels'
import GameCover from './GameCover.vue'
import { CoverSize } from './variants'
import { coverKinds } from '@/components/library/libraryView'

const props = defineProps<{ game: LibraryGame }>()
const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()

const kinds = computed(() => coverKinds(props.game))
const choice = ref<CoverKind>(props.game.cover)
const busy = ref(false)
const error = ref<string | null>(null)

// Previews: the generated cover always, the others when the game already shows them.
const generated = computed<LibraryGame>(() => ({
  ...props.game,
  cover: 'generated',
  coverUrl: null,
}))
const iconUrl = computed(() => (props.game.cover === 'icon' ? props.game.coverUrl : null))
const userUrl = computed(() => (props.game.cover === 'user' ? props.game.coverUrl : null))
/** «Immagine tua» before the user has one: an empty card inviting to add it. */
const isEmpty = (k: CoverKind) => k === 'user' && !userUrl.value

function choose(value: unknown) {
  const k = kinds.value.find(x => x === value)
  if (k) choice.value = k
}

async function apply() {
  if (busy.value) return
  busy.value = true
  error.value = null
  try {
    if (await emu.library.setCover(props.game.id, choice.value)) emit('close')
  } catch (e) {
    error.value = errorText(e)
  } finally {
    busy.value = false
  }
}

const keepOpen = (e: Event) => e.preventDefault()
</script>

<template>
  <Dialog :open="true" @update:open="open => open || emit('close')">
    <DialogContent
      :size="DialogSize.Wide"
      :aria-label="t('library.cover.title', { title: game.title })"
      :aria-describedby="undefined"
      @interact-outside="keepOpen"
    >
      <DialogTitle as="div" :size="DialogTitleSize.Heading">
        {{ t('library.cover.title', { title: game.title }) }}
      </DialogTitle>
      <RadioGroup
        :model-value="choice"
        class="grid grid-cols-3 gap-12"
        :aria-label="t('library.cover.title', { title: game.title })"
        @update:model-value="choose"
      >
        <RadioGroupItem v-for="k in kinds" :key="k" :value="k">
          <Card
            as="button"
            :variant="CardVariant.PictureChoice"
            :selected="choice === k"
            :data-empty="isEmpty(k) ? '' : undefined"
            class="gap-8"
            :data-nav-initial="choice === k ? '' : undefined"
          >
            <GameCover
              v-if="k === 'generated'"
              :size="CoverSize.Choice"
              :game="generated"
              :badge="false"
            />
            <span
              v-else
              class="relative grid aspect-2/3 place-items-center overflow-hidden rounded-6 text-ink-muted"
              :class="{ 'bg-surface-raised': !isEmpty(k) }"
            >
              <template v-if="k === 'icon'">
                <img v-if="iconUrl" class="pixelated size-48" :src="iconUrl" alt="" />
                <span v-else class="size-48 rounded-6" :style="{ background: game.colour }"></span>
              </template>
              <template v-else>
                <img
                  v-if="userUrl"
                  class="absolute inset-0 size-full object-cover"
                  :src="userUrl"
                  alt=""
                />
                <Plus v-else :size="20" :stroke-width="1.75" aria-hidden="true" />
              </template>
            </span>
            <span class="font-medium">{{ t(`library.cover.${k}`) }}</span>
            <span class="text-caption text-ink-muted">{{ t(`library.cover.${k}Desc`) }}</span>
          </Card>
        </RadioGroupItem>
      </RadioGroup>
      <p v-if="error" class="m-0 text-danger" role="alert">{{ error }}</p>
      <div class="flex justify-end gap-8">
        <Button @click="emit('close')">{{ t('library.cover.cancel') }}</Button>
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.Medium"
          :disabled="busy"
          @click="apply"
        >
          {{ t('library.cover.use') }}
        </Button>
      </div>
    </DialogContent>
  </Dialog>
</template>
