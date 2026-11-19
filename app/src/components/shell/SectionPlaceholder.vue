<script setup lang="ts">
// A section that has no screen yet (Cloud): its name and one line saying it is coming.
import { useI18n } from 'vue-i18n'
import type { Section } from './sections'
import { Heading } from '@/components/ui/heading'

defineProps<{ section: Section }>()

const { t } = useI18n()
</script>

<template>
  <section class="flex flex-col gap-8">
    <Heading as="h1">{{ t(`shell.${section}`) }}</Heading>
    <p class="m-0 text-ink-muted">{{ t('placeholder.line') }}</p>
  </section>
</template>
