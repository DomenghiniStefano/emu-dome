<script setup lang="ts">
// The main screen's foot: «Chiudi il gioco» far from «Riprendi il gioco», bottom left on desktop,
// beside it on the phone. Between them, the keys that open the search (X01, #32) for the input
// in use (none on touch, nor in the phone's sheet).
import { useI18n } from 'vue-i18n'
import { Play, Power } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Kbd, KbdSize, KbdVariant } from '@/design-system/components/kbd'

defineProps<{
  /** The key that resumes with the input in use, or null. */
  hint: string | null
  /** The keys that open the search with the input in use (none: no hint). */
  searchKeys?: string[]
  touch?: boolean
}>()
defineEmits<{ close: []; resume: [] }>()

const { t } = useI18n()
</script>

<template>
  <footer
    class="flex items-center justify-between gap-8 pt-4 touch:grid touch:grid-cols-[auto_minmax(0,1fr)] touch:gap-8"
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
    <span
      v-if="!touch && searchKeys?.length"
      class="inline-flex items-center gap-6 text-caption text-ink-muted"
    >
      <Kbd v-for="key in searchKeys" :key="key">{{ key }}</Kbd>
      {{ t('search.title') }}
    </span>
    <Button
      v-if="touch"
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.MenuMain"
      :icon="Play"
      icon-filled
      @click="$emit('resume')"
    >
      {{ t('qm.resumeShort') }}
    </Button>
    <Button
      v-else
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.MenuMain"
      :icon="Play"
      icon-filled
      @click="$emit('resume')"
    >
      {{ t('qm.resume') }}
      <Kbd v-if="hint" :variant="KbdVariant.OnAccent" :size="KbdSize.Small" class="ml-4">
        {{ hint }}
      </Kbd>
    </Button>
  </footer>
</template>
