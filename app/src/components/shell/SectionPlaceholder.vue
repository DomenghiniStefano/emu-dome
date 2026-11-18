<script setup lang="ts">
// A section that has no screen yet (Cloud): its name and one line saying it is coming.
import { useI18n } from 'vue-i18n'
import type { Section } from '../../sections'

defineProps<{ section: Section }>()

const { t } = useI18n()
</script>

<template>
  <section class="flex flex-col gap-8">
    <h1 class="m-0 text-title">{{ t(`shell.${section}`) }}</h1>
    <p class="m-0 text-ink-muted">{{ t('placeholder.line') }}</p>
  </section>
</template>
