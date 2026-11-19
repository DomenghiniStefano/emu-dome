<script setup lang="ts">
// The main screen's foot: «Chiudi il gioco» far from «Riprendi il gioco», bottom left on desktop,
// beside it on the phone.
import { useI18n } from 'vue-i18n'
import { Play, Power } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd, KbdSize, KbdVariant } from '@/components/ui/kbd'

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
    class="flex items-center justify-between gap-12 pt-4 touch:grid touch:grid-cols-[auto_minmax(0,1fr)] touch:gap-8"
  >
    <Button
      :variant="ButtonVariant.Danger"
      :size="ButtonSize.Menu"
      class="min-w-0"
      @click="$emit('close')"
    >
      <Power v-if="!touch" :size="18" :stroke-width="1.75" aria-hidden="true" />
      {{ t('qm.close') }}
    </Button>
    <Button
      v-if="touch"
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.MenuMain"
      @click="$emit('resume')"
    >
      {{ t('qm.resumeShort') }}
    </Button>
    <Button
      v-else
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.MenuMain"
      @click="$emit('resume')"
    >
      <Play :size="18" fill="currentColor" :stroke-width="0" aria-hidden="true" />
      {{ t('qm.resume') }}
      <Kbd v-if="hint" :variant="KbdVariant.OnAccent" :size="KbdSize.Small" class="ml-4">
        {{ hint }}
      </Kbd>
    </Button>
  </footer>
</template>
