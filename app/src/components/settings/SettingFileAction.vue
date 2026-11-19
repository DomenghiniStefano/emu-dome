<script setup lang="ts">
// The button of a file or action row («Scegli un’immagine…», «Esporta…»). Beside it, a file row
// says which file is chosen, or that none is. A row with no backend yet shows its button disabled, with the reason
// on hover.
import { useI18n } from 'vue-i18n'
import { Button, ButtonFade, ButtonVariant } from '@/components/ui/button'
import { Badge, BadgeSize, BadgeVariant } from '@/components/ui/badge'

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
  <Badge v-if="fileState" :variant="BadgeVariant.Quiet" :size="BadgeSize.Pill">
    {{ fileState }}
  </Badge>
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
