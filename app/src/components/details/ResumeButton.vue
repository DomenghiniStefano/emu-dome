<script setup lang="ts">
// L04's main action: «Riprendi», or «Gioca» when nothing is saved. Large beside the cover on the
// desktop, full width under it on the phone. Faded, not disabled, when the game file is missing:
// a press still says why (GameDetails `play`). The page's first focus.
import { useI18n } from 'vue-i18n'
import { Play } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

defineProps<{
  /** The game file is where the library expects it. */
  available: boolean
  /** Nothing saved yet: «Gioca». */
  fresh: boolean
}>()

const { t } = useI18n()
</script>

<template>
  <Button
    :variant="ButtonVariant.Primary"
    :size="ButtonSize.Large"
    data-nav-initial
    :aria-disabled="!available || undefined"
  >
    <Play
      :size="16"
      class="touch:size-18"
      fill="currentColor"
      :stroke-width="0"
      aria-hidden="true"
    />
    {{ t(fresh ? 'library.hero.play' : 'library.hero.resume') }}
  </Button>
</template>
