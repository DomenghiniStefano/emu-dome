<script setup lang="ts">
// S04 Controller (only while one is connected): the A/B layout (`pad.ab`) as two cards, and a
// press test that says what each button does in the game. The only step with «Salta».
import { useI18n } from 'vue-i18n'
import { Card, CardVariant } from '@/components/ui/card'
import type { PadInfo } from '../../emu'
import { gamepadName } from '../../i18n/labels'
import type { AbLayout } from '../../composables/usePadTester'
import StepLayout from './StepLayout.vue'
import StepTitle from './StepTitle.vue'

defineProps<{
  phone: boolean
  pad: PadInfo
  ab: AbLayout
  /** The last press: what it is on the controller and what the game gets (null: unused). */
  pressed: { which: string; button: string | null } | null
}>()
const emit = defineEmits<{ ab: [id: AbLayout] }>()
const { t } = useI18n()

const LAYOUTS: AbLayout[] = ['nintendo', 'printed']
</script>

<template>
  <StepLayout wide :phone="phone">
    <StepTitle :phone="phone">{{ t('firstRun.controller.title') }}</StepTitle>
    <p class="m-0 text-lead text-ink-muted">
      {{ t('firstRun.controller.lead', { name: gamepadName(pad) }) }}
    </p>
    <div
      class="grid gap-12"
      :class="phone ? 'grid-cols-1' : 'grid-cols-2'"
      role="radiogroup"
      :aria-label="t('settings.pad.ab.name')"
    >
      <Card
        v-for="id in LAYOUTS"
        :key="id"
        as="button"
        :variant="CardVariant.LayoutChoice"
        :selected="ab === id"
        class="gap-6"
        role="radio"
        :aria-checked="ab === id"
        :data-nav-initial="id === 'nintendo' ? '' : undefined"
        @click="emit('ab', id)"
      >
        <span class="font-semibold">{{ t(`settings.pad.ab.option.${id}`) }}</span>
        <span class="text-caption text-ink-muted">{{ t(`firstRun.controller.${id}`) }}</span>
      </Card>
    </div>
    <div
      class="flex min-h-[70px] items-center gap-12 rounded-10 border border-line bg-surface p-14"
      aria-live="polite"
    >
      <span
        v-if="pressed?.button"
        class="grid size-control-lg flex-none place-items-center rounded-full bg-accent-soft font-semibold text-accent-text"
      >
        {{ pressed.button }}
      </span>
      <span class="text-ink-muted">
        <template v-if="!pressed">{{ t('firstRun.controller.press') }}</template>
        <i18n-t v-else-if="pressed.button" keypath="firstRun.controller.pressed" scope="global">
          <template #which>{{ pressed.which }}</template>
          <template #button>
            <strong class="text-ink">{{ pressed.button }}</strong>
          </template>
        </i18n-t>
        <template v-else>
          {{ t('firstRun.controller.unused', { which: pressed.which }) }}
        </template>
      </span>
    </div>
  </StepLayout>
</template>
