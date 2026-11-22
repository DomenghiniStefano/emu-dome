<script setup lang="ts">
// «Scegli uno screenshot» (Trello #85, the owner's mockup, section 2): this game's screenshots,
// newest first, as a grid to choose from; «Indietro» / «Continua». A step of CoverPicker.
// Controller: its own nav scope inside the dialog's, the newest screenshot (or the one chosen
// before) focused first; A chooses, B goes back.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ArrowRight, ChevronLeft } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import { DialogTitle, DialogTitleSize } from '@/components/ui/dialog'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import type { GameScreenshot } from '@/emu'
import { dateTimeLabel } from '@/i18n/labels'
import { useNavScope } from '@/nav'

const props = defineProps<{ shots: GameScreenshot[]; chosen: string | null }>()
const emit = defineEmits<{ back: []; next: [GameScreenshot] }>()

const { t } = useI18n()
const root = ref<HTMLElement | null>(null)
const choice = ref<string | null>(props.chosen ?? props.shots[0]?.name ?? null)
useNavScope(root, { back: () => emit('back'), autofocus: true })

function choose(value: unknown) {
  const shot = props.shots.find(s => s.name === value)
  if (shot) choice.value = shot.name
}
function next() {
  const shot = props.shots.find(s => s.name === choice.value)
  if (shot) emit('next', shot)
}
const takenAt = (s: GameScreenshot) =>
  t('library.cover.shots.taken', { date: dateTimeLabel(s.takenAt) })
</script>

<template>
  <div ref="root" class="flex flex-col gap-12">
    <DialogTitle as="div" :size="DialogTitleSize.Heading">
      {{ t('library.cover.shots.title') }}
    </DialogTitle>
    <RadioGroup
      :model-value="choice ?? undefined"
      class="grid grid-cols-3 gap-8"
      :aria-label="t('library.cover.shots.title')"
      @update:model-value="choose"
    >
      <RadioGroupItem v-for="s in shots" :key="s.name" :value="s.name">
        <Card
          as="button"
          :variant="CardVariant.PictureChoice"
          :selected="choice === s.name"
          :aria-label="takenAt(s)"
          :data-nav-initial="choice === s.name ? '' : undefined"
        >
          <img
            class="aspect-3/2 w-full rounded-badge bg-letterbox object-contain"
            :src="s.url"
            alt=""
            loading="lazy"
          />
        </Card>
      </RadioGroupItem>
    </RadioGroup>
    <div class="flex justify-between gap-8">
      <Button :size="ButtonSize.Medium" :icon="ChevronLeft" @click="emit('back')">
        {{ t('library.cover.back') }}
      </Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        :icon="ArrowRight"
        :disabled="!choice"
        @click="next"
      >
        {{ t('library.cover.continue') }}
      </Button>
    </div>
  </div>
</template>
