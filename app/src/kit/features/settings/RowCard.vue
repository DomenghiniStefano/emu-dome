<script setup lang="ts">
// The card a settings page puts its rows on (SettingsSection without the heading), as wide as the
// page's column (the whole page, as the settings page now is), so a row on the Kit page is seen at its real width.
import { Card, CardVariant } from '@/components/ui/card'
</script>

<template>
  <Card :variant="CardVariant.List" class="w-full">
    <slot />
  </Card>
</template>
