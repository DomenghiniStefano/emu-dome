<script setup lang="ts">
// The rows of a quick menu page in one rounded list (G10, G16).
import { Card, CardVariant } from '@/design-system/components/card'
</script>

<template>
  <Card :variant="CardVariant.List" class="flex-none">
    <slot />
  </Card>
</template>
