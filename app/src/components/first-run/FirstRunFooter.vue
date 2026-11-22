<script setup lang="ts">
// The first run's footer: «Indietro» on the left, «Salta» (S04 only) and the main action on the
// right. On the phone the main action comes first, full width, then «Indietro» and «Salta» side
// by side (ButtonSize.Large and Footer have the phone's sizes). The footer's height 96 is the
// board's own. Each with its icon before the words (#104): the main action an arrow, the last
// step's the library it leads to; «Indietro» a chevron, «Salta» skip.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ArrowRight, ChevronLeft, Library, SkipForward } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

const props = defineProps<{
  phone: boolean
  /** «Indietro» is there from the second step on. */
  canGoBack: boolean
  /** S04: «Salta» is there. */
  skippable: boolean
  /** The main action's label: «Inizia», «Continua», «Fine». */
  mainLabel: string
  /** The last step: the main action leads to the library. */
  last?: boolean
}>()
const emit = defineEmits<{ next: []; back: [] }>()
const { t } = useI18n()

const mainIcon = computed(() => (props.last ? Library : ArrowRight))
</script>

<template>
  <footer v-if="phone" class="flex flex-none flex-col items-stretch gap-8 px-20 pt-12 pb-32">
    <Button
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.Large"
      :icon="mainIcon"
      data-nav-initial
      @click="emit('next')"
    >
      {{ mainLabel }}
    </Button>
    <div class="flex flex-1 items-center justify-between gap-8">
      <Button
        v-if="canGoBack"
        :size="ButtonSize.Footer"
        :icon="ChevronLeft"
        class="flex-1"
        @click="emit('back')"
      >
        {{ t('firstRun.back') }}
      </Button>
      <Button
        v-if="skippable"
        :size="ButtonSize.Footer"
        :icon="SkipForward"
        class="flex-1"
        @click="emit('next')"
      >
        {{ t('firstRun.skip') }}
      </Button>
    </div>
  </footer>
  <footer v-else class="flex h-footer flex-none items-center px-24">
    <div class="flex flex-1 items-center justify-between gap-8">
      <Button v-if="canGoBack" :size="ButtonSize.Footer" :icon="ChevronLeft" @click="emit('back')">
        {{ t('firstRun.back') }}
      </Button>
      <span v-else></span>
      <div class="flex gap-8">
        <Button
          v-if="skippable"
          :size="ButtonSize.Footer"
          :icon="SkipForward"
          @click="emit('next')"
        >
          {{ t('firstRun.skip') }}
        </Button>
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.Large"
          :icon="mainIcon"
          data-nav-initial
          @click="emit('next')"
        >
          {{ mainLabel }}
        </Button>
      </div>
    </div>
  </footer>
</template>
