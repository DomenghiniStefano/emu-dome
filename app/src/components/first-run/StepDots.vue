<script setup lang="ts">
// Where the first run is: one dot per step, the current one long, the ones done half accent.
// On the phone they sit lower, above the step. Sizes 72 and 88 are the boards' own.
import { useI18n } from 'vue-i18n'
import type { FirstRunStep } from '../../emu'

defineProps<{ steps: FirstRunStep[]; index: number; phone: boolean }>()
const { t } = useI18n()

const dotClass = (i: number, index: number) =>
  i === index ? 'w-24 bg-accent-text' : i < index ? 'w-8 bg-dot-done' : 'w-8 bg-dot'
</script>

<template>
  <ol
    class="m-0 flex flex-none list-none justify-center gap-8 p-0"
    :class="phone ? 'h-[88px] items-end pb-12' : 'h-[72px] items-center'"
    :aria-label="t('firstRun.steps')"
  >
    <li
      v-for="(s, i) in steps"
      :key="s"
      class="h-8 rounded-full"
      :class="dotClass(i, index)"
      :aria-label="t(`firstRun.step.${s}`)"
      :aria-current="i === index ? 'step' : undefined"
    ></li>
  </ol>
</template>
