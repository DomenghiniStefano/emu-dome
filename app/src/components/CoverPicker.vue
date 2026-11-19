<script setup lang="ts">
// «Scegli la copertina» (board L12, Stati): generated, the ROM icon (only when the ROM has one),
// or the user's image (the backend opens the image picker; cancelling it keeps this open).
// Controller / keyboard: its own nav scope, B / Esc cancel.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Plus } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import { emu, type CoverKind, type LibraryGame } from '../emu'
import { errorText } from '../i18n/labels'
import { useNavScope } from '../nav'
import GameCover from './GameCover.vue'
import { CoverSize } from './cover/variants'
import { coverKinds } from './library/libraryView'

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

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true, escape: true, back: () => emit('close') })
</script>

<template>
  <!-- The veil's 8px blur and the dialog's 640px width are not design system values yet
       (Cowork, spec Decision 5). -->
  <div class="fixed inset-0 z-menu grid place-items-center bg-scrim p-16 backdrop-blur-dim">
    <div
      ref="root"
      class="glass-thick flex max-h-full w-panel max-w-full flex-col gap-14 overflow-auto p-24"
      role="dialog"
      aria-modal="true"
      :aria-label="t('library.cover.title', { title: game.title })"
    >
      <div class="text-heading">{{ t('library.cover.title', { title: game.title }) }}</div>
      <div
        class="grid grid-cols-3 gap-12"
        role="radiogroup"
        :aria-label="t('library.cover.title', { title: game.title })"
      >
        <Card
          v-for="k in kinds"
          :key="k"
          as="button"
          :variant="CardVariant.PictureChoice"
          :selected="choice === k"
          :data-empty="isEmpty(k) ? '' : undefined"
          class="gap-8"
          role="radio"
          :aria-checked="choice === k"
          :data-nav-initial="choice === k ? '' : undefined"
          @click="choice = k"
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
      </div>
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
    </div>
  </div>
</template>
