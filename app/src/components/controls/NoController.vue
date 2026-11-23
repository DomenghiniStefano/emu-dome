<script setup lang="ts">
// E05: no controller connected. The keyboard can be chosen meanwhile; on the phone with no
// physical keyboard used yet (#118) the touch controls are the way to play instead.
import { useI18n } from 'vue-i18n'
import { Gamepad2, Keyboard } from '@lucide/vue'
import { useKeyboardKeys } from '@/composables/useKeyboardKeys'
import { Button, ButtonSize } from '@/design-system/components/button'
import { EmptyState } from '@/design-system/components/empty-state'
import { Heading } from '@/design-system/components/heading'
import { SectionPage } from '@/design-system/components/section-page'

const emit = defineEmits<{ keyboard: [] }>()
const { t } = useI18n()
const keyboardKeys = useKeyboardKeys()
</script>

<template>
  <SectionPage as="section">
    <Heading as="h1">{{ t('shell.controls') }}</Heading>
    <EmptyState
      as="div"
      :icon="Gamepad2"
      :title="t('controls.e05.title')"
      :text="keyboardKeys ? t('controls.e05.lead') : t('controls.e05.leadTouch')"
    >
      <Button
        v-if="keyboardKeys"
        :size="ButtonSize.Medium"
        :icon="Keyboard"
        data-nav-initial
        @click="emit('keyboard')"
      >
        {{ t('controls.e05.keyboard') }}
      </Button>
    </EmptyState>
  </SectionPage>
</template>
