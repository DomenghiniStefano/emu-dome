<script setup lang="ts">
// G06: how to open the quick menu, at the top over the game, the first few games. It fades out
// (600 ms, a value waiting for a design decision: spec Decision 5). On the phone it names only
// the input that is there, and does not show with neither a keyboard nor a controller: the touch
// controls' menu button is on screen (quickMenuHint.ts, #118).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import { uiMode } from '@/composables/useUiMode'
import { keyboardSeen } from '@/lib/inputMode'
import { Kbd } from '@/design-system/components/kbd'
import { quickMenuHintKeys } from './quickMenuHint'

defineProps<{ shown: boolean }>()
const { t } = useI18n()
const { touchDensity } = useDeviceLayout()
const keys = computed(() =>
  quickMenuHintKeys(touchDensity.value, keyboardSeen.value, uiMode.pad !== null),
)
</script>

<template>
  <Transition leave-active-class="transition-opacity duration-hint" leave-to-class="opacity-0">
    <div
      v-if="shown && keys"
      class="glass-thick pointer-events-none fixed top-12 left-1/2 z-overlay flex -translate-x-1/2 items-center gap-6 rounded-card px-10 py-6 text-ink-muted"
      role="status"
    >
      <Kbd v-if="keys.esc">{{ t('flow.hintEsc') }}</Kbd>
      <span v-if="keys.esc && keys.home">{{ t('flow.hintOr') }}</span>
      <Kbd v-if="keys.home">{{ t('flow.hintHome') }}</Kbd>
      <span>{{ t('flow.hintText') }}</span>
    </div>
  </Transition>
</template>
