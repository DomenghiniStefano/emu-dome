<script setup lang="ts">
// S01 Welcome: the mark, and the language (`general.lang`, «Tutti i giochi»). No «Indietro».
import { useI18n } from 'vue-i18n'
import SettingSelect from '../SettingSelect.vue'
import StepLayout from './StepLayout.vue'
import StepTitle from './StepTitle.vue'

defineProps<{ phone: boolean; lang: string; langOptions: { id: string; text: string }[] }>()
const emit = defineEmits<{ lang: [id: string] }>()
const { t } = useI18n()
</script>

<template>
  <StepLayout centred :phone="phone">
    <!-- The mark's radius (20) waits for Cowork (spec Decision 5). -->
    <div class="size-[72px] rounded-[20px] bg-logo shadow-glow-lg" aria-hidden="true"></div>
    <StepTitle :phone="phone">{{ t('firstRun.welcome.title') }}</StepTitle>
    <p class="m-0 max-w-[520px] text-lead text-ink-muted">{{ t('firstRun.welcome.lead') }}</p>
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
