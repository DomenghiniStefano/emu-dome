<script setup lang="ts">
// The first run's footer: «Indietro» on the left, «Salta» (S04 only) and the main action on the
// right. On the phone the main action comes first, full width, then «Indietro» and «Salta» side
// by side (ButtonSize.Large and Footer have the phone's sizes). The footer's height 96 is the
// board's own.
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

defineProps<{
  phone: boolean
  /** «Indietro» is there from the second step on. */
  canGoBack: boolean
  /** S04: «Salta» is there. */
  skippable: boolean
  /** The main action's label: «Inizia», «Continua», «Fine». */
  mainLabel: string
}>()
const emit = defineEmits<{ next: []; back: [] }>()
const { t } = useI18n()
</script>

<template>
  <footer v-if="phone" class="flex flex-none flex-col items-stretch gap-8 px-20 pt-12 pb-32">
    <Button
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.Large"
      data-nav-initial
      @click="emit('next')"
    >
      {{ mainLabel }}
    </Button>
    <div class="flex flex-1 items-center justify-between gap-8">
      <Button v-if="canGoBack" :size="ButtonSize.Footer" class="flex-1" @click="emit('back')">
        {{ t('firstRun.back') }}
      </Button>
      <Button v-if="skippable" :size="ButtonSize.Footer" class="flex-1" @click="emit('next')">
        {{ t('firstRun.skip') }}
      </Button>
    </div>
  </footer>
  <footer v-else class="flex h-footer flex-none items-center px-48">
    <div class="flex flex-1 items-center justify-between gap-8">
      <Button v-if="canGoBack" :size="ButtonSize.Footer" @click="emit('back')">
        {{ t('firstRun.back') }}
      </Button>
      <span v-else></span>
      <div class="flex gap-8">
        <Button v-if="skippable" :size="ButtonSize.Footer" @click="emit('next')">
          {{ t('firstRun.skip') }}
        </Button>
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.Large"
          data-nav-initial
          @click="emit('next')"
        >
          {{ mainLabel }}
        </Button>
      </div>
    </div>
  </footer>
</template>
