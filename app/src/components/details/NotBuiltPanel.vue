<script setup lang="ts">
// A tab that is not built yet (L08 Codici trucco, L09 Patch e traduzioni): what it will do, and
// that it arrives later (X05, the reason).
import { useI18n } from 'vue-i18n'
import { Card } from '@/components/ui/card'

defineProps<{ tab: 'cheats' | 'patches' }>()

const { t } = useI18n()
</script>

<template>
  <Card class="gap-8">
    <p class="m-0">{{ t(`details.${tab}.lead`) }}</p>
    <p class="m-0 text-ink-muted">{{ t('details.notYet') }}</p>
  </Card>
</template>
