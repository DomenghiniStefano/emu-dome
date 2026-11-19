<script setup lang="ts">
// S01 Welcome: the mark, and the language (`general.lang`, «Tutti i giochi»). No «Indietro».
import { useI18n } from 'vue-i18n'
import SettingSelect from '../SettingSelect.vue'
import StepLayout from './StepLayout.vue'
import { Heading, HeadingVariant } from '@/components/ui/heading'

defineProps<{ lang: string; langOptions: { id: string; text: string }[] }>()
const emit = defineEmits<{ lang: [id: string] }>()
const { t } = useI18n()
</script>

<template>
  <StepLayout centred>
    <div class="size-logo rounded-20 bg-logo shadow-glow-lg" aria-hidden="true"></div>
    <Heading as="h1" :variant="HeadingVariant.Display">{{ t('firstRun.welcome.title') }}</Heading>
    <p class="m-0 max-w-lead text-lead text-ink-muted">{{ t('firstRun.welcome.lead') }}</p>
    <div class="flex items-center gap-8 text-ink-muted">
      <span>{{ t('firstRun.welcome.language') }}</span>
      <SettingSelect
        :label="t('settings.general.lang.name')"
        :value="lang"
        :options="langOptions"
        @pick="id => emit('lang', id)"
      />
    </div>
  </StepLayout>
</template>
