<script setup lang="ts">
// The button of a file or action row («Scegli un’immagine…», «Esporta…»). Beside it, a file row
// says which file is chosen, or that none is. A row with no backend yet shows its button disabled, with the reason
// on hover.
import { useI18n } from 'vue-i18n'
import { Button, ButtonFade, ButtonVariant } from '@/components/ui/button'

defineProps<{
  text: string
  /** «crt.slangp», «Nessun file scelto», or '' to show nothing. */
  fileState: string
  notYet: boolean
  disabled: boolean
}>()
const emit = defineEmits<{ choose: [] }>()
const { t } = useI18n()
</script>

<template>
  <!-- line height 20: off the type scale (Cowork) -->
  <span
    v-if="fileState"
    class="rounded-full bg-surface-hover px-8 text-caption leading-file whitespace-nowrap text-ink-muted"
  >
    {{ fileState }}
  </span>
  <template v-if="text">
    <Button
      v-if="notYet"
      :variant="ButtonVariant.Soft"
      :fade="ButtonFade.None"
      disabled
      :title="t('settingsUi.notYet')"
    >
      {{ text }}
    </Button>
    <Button
      v-else
      :variant="ButtonVariant.Soft"
      :fade="ButtonFade.None"
      :disabled="disabled"
      @click="emit('choose')"
    >
      {{ text }}
    </Button>
  </template>
</template>
