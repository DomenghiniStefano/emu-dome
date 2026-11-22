<script setup lang="ts">
// The card a settings page puts its rows on (SettingsSection without the heading), as wide as the
// page's column, so a row on the Kit page is seen at its real width.
import { Card, CardVariant } from '@/components/ui/card'
</script>

<template>
  <Card :variant="CardVariant.List" class="w-page max-w-full">
    <slot />
  </Card>
</template>
