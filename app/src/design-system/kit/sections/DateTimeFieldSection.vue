<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { DateTimeField, DateTimeFieldVariant } from '@/design-system/components/date-time-field'
import { settingDateTimeLabel } from '@/i18n/labels'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'

const { locale } = useI18n()
const value = ref('2004-01-01T00:00')
</script>

<template>
  <KitSection title="DateTimeField">
    <KitState label="Field (settings rows): A opens the segments, written on closing">
      <div class="flex items-center gap-12">
        <DateTimeField
          v-model="value"
          :variant="DateTimeFieldVariant.Field"
          label="Date and time"
          :text="settingDateTimeLabel(value)"
          :min-year="2000"
          :max-year="2099"
          :locale="locale"
        />
        <span class="text-caption text-ink-faint">value {{ value }}</span>
      </div>
    </KitState>
    <KitState label="Disabled">
      <DateTimeField
        model-value="2024-02-29T07:05"
        label="Disabled"
        :text="settingDateTimeLabel('2024-02-29T07:05')"
        :min-year="2000"
        :max-year="2099"
        :locale="locale"
        disabled
      />
    </KitState>
  </KitSection>
</template>
