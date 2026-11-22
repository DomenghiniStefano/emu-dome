<script setup lang="ts">
// L12 «Scegli la copertina», its first step (the owner's mockup 2026-11-21, section 2): generated,
// the ROM icon (only when the ROM has one), «Da uno screenshot» (this game's count, disabled with
// the reason when it has none) and «Immagine tua», as a RadioGroup of picture cards (the arrows
// along the row choose). A step of CoverPicker.
// Controller: its own nav scope inside the dialog's, the chosen cover focused first; B / Esc
// cancel.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Plus } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import { DialogTitle, DialogTitleSize } from '@/components/ui/dialog'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import type { GameScreenshot, LibraryGame } from '@/emu'
import { useNavScope } from '@/nav'
import GameCover from './GameCover.vue'
import { CoverSize } from './variants'
import { coverChoices, type CoverChoice } from './coverChoice'

const props = defineProps<{
  game: LibraryGame
  /** This game's screenshots; null while they are read. */
  shots: GameScreenshot[] | null
  busy: boolean
  error: string | null
}>()
const choice = defineModel<CoverChoice>({ required: true })
const emit = defineEmits<{ cancel: []; next: [] }>()

const { t } = useI18n()
const root = ref<HTMLElement | null>(null)
useNavScope(root, { back: () => emit('cancel'), autofocus: true })

const choices = computed(() => coverChoices(props.game))
const noShots = computed(() => props.shots !== null && props.shots.length === 0)
const disabled = (c: CoverChoice) => c === 'screenshot' && (props.shots === null || noShots.value)

// Previews: the generated cover always, the others when the game already shows them, the newest
// screenshot.
const generated = computed<LibraryGame>(() => ({
  ...props.game,
  cover: 'generated',
  coverUrl: null,
}))
const iconUrl = computed(() => (props.game.cover === 'icon' ? props.game.coverUrl : null))
const userUrl = computed(() => (props.game.cover === 'user' ? props.game.coverUrl : null))
const shotUrl = computed(() => props.shots?.[0]?.url ?? null)
/** «Immagine tua» before the user has one: an empty card inviting to add it. */
const isEmpty = (c: CoverChoice) => c === 'user' && !userUrl.value

function choose(value: unknown) {
  const c = choices.value.find(x => x === value)
  if (c && !disabled(c)) choice.value = c
}
const desc = (c: CoverChoice) => {
  if (c !== 'screenshot') return t(`library.cover.${c}Desc`)
  if (!props.shots?.length) return t('library.cover.screenshotNone')
  return t('library.cover.screenshotDesc', { count: props.shots.length }, props.shots.length)
}
/** Generated and icon apply at once; a screenshot and an image go on to «Adatta la copertina». */
const nextLabel = computed(() =>
  choice.value === 'screenshot' || choice.value === 'user'
    ? t('library.cover.continue')
    : t('library.cover.use'),
)
</script>

<template>
  <div ref="root" class="flex flex-col gap-12">
    <DialogTitle as="div" :size="DialogTitleSize.Heading">
      {{ t('library.cover.title', { title: game.title }) }}
    </DialogTitle>
    <RadioGroup
      :model-value="choice"
      class="grid gap-8"
      :class="choices.length > 3 ? 'grid-cols-4' : 'grid-cols-3'"
      :aria-label="t('library.cover.title', { title: game.title })"
      @update:model-value="choose"
    >
      <RadioGroupItem v-for="c in choices" :key="c" :value="c" :disabled="disabled(c)">
        <Card
          as="button"
          :variant="CardVariant.PictureChoice"
          :selected="choice === c"
          :disabled="disabled(c)"
          :data-empty="isEmpty(c) ? '' : undefined"
          class="gap-6"
          :data-nav-initial="choice === c ? '' : undefined"
        >
          <GameCover
            v-if="c === 'generated'"
            :size="CoverSize.Choice"
            :game="generated"
            :badge="false"
          />
          <span
            v-else
            class="relative grid aspect-2/3 place-items-center overflow-hidden rounded-control text-ink-muted"
            :class="{ 'bg-surface-raised': !isEmpty(c) }"
          >
            <template v-if="c === 'icon'">
              <img v-if="iconUrl" class="pixelated size-48" :src="iconUrl" alt="" />
              <span
                v-else
                class="size-48 rounded-control"
                :style="{ background: game.colour }"
              ></span>
            </template>
            <template v-else-if="c === 'screenshot'">
              <template v-if="shotUrl">
                <img
                  class="absolute inset-0 size-full object-cover blur-dim"
                  :src="shotUrl"
                  alt=""
                />
                <span class="absolute inset-0 bg-game-scrim"></span>
                <img class="absolute w-full object-contain" :src="shotUrl" alt="" />
              </template>
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
          <span class="font-medium">{{ t(`library.cover.${c}`) }}</span>
          <span class="text-caption text-ink-muted">{{ desc(c) }}</span>
        </Card>
      </RadioGroupItem>
    </RadioGroup>
    <p v-if="error" class="m-0 text-danger" role="alert">{{ error }}</p>
    <div class="flex justify-end gap-8">
      <Button :size="ButtonSize.Medium" @click="emit('cancel')">{{
        t('library.cover.cancel')
      }}</Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        :disabled="busy"
        @click="emit('next')"
      >
        {{ nextLabel }}
      </Button>
    </div>
  </div>
</template>
