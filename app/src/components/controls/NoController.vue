<script setup lang="ts">
// E05: no controller connected. The keyboard can be chosen meanwhile.
import { useI18n } from 'vue-i18n'
import { Gamepad2, Keyboard } from '@lucide/vue'
import { Button, ButtonSize } from '@/components/ui/button'
import { EmptyState } from '@/components/ui/empty-state'
import { Heading } from '@/components/ui/heading'
import { SectionPage } from '@/components/ui/section-page'

const emit = defineEmits<{ keyboard: [] }>()
const { t } = useI18n()
</script>

<template>
  <SectionPage as="section">
    <Heading as="h1">{{ t('shell.controls') }}</Heading>
    <EmptyState
      as="div"
      :icon="Gamepad2"
      :title="t('controls.e05.title')"
      :text="t('controls.e05.lead')"
    >
      <Button :size="ButtonSize.Medium" :icon="Keyboard" data-nav-initial @click="emit('keyboard')">
        {{ t('controls.e05.keyboard') }}
      </Button>
    </EmptyState>
  </SectionPage>
</template>
