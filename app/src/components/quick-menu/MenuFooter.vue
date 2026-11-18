<script setup lang="ts">
// The main screen's foot: «Chiudi il gioco» far from «Riprendi il gioco», bottom left on desktop,
// beside it on the phone.
import { useI18n } from 'vue-i18n'
import { Play, Power } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import KeyHint from './KeyHint.vue'

defineProps<{
  /** The key that resumes with the input in use, or null. */
  hint: string | null
  touch?: boolean
}>()
defineEmits<{ close: []; resume: [] }>()

const { t } = useI18n()
</script>

<template>
  <footer
    class="items-center justify-between pt-4"
    :class="touch ? 'grid grid-cols-[auto_minmax(0,1fr)] gap-8' : 'flex gap-12'"
  >
    <Button
      :variant="ButtonVariant.Danger"
      :size="touch ? ButtonSize.MenuTouch : ButtonSize.Menu"
      class="min-w-0"
      @click="$emit('close')"
    >
      <Power v-if="!touch" :size="18" :stroke-width="1.75" aria-hidden="true" />
      {{ t('qm.close') }}
    </Button>
    <Button
      v-if="touch"
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.MenuMainTouch"
      @click="$emit('resume')"
    >
      {{ t('qm.resumeShort') }}
    </Button>
    <Button
      v-else
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.MenuMain"
      class="w-[260px]"
      @click="$emit('resume')"
    >
      <Play :size="18" fill="currentColor" :stroke-width="0" aria-hidden="true" />
      {{ t('qm.resume') }}
      <KeyHint v-if="hint" :label="hint" on-accent />
    </Button>
  </footer>
</template>
