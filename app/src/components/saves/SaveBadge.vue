<script setup lang="ts">
// A short status beside a name: «Salvataggio illeggibile» on a game of the list (V01, warning),
// «Più recente» on a side of a conflict (V06, neutral: the choice stays the user's, never the
// accent).
import type { SaveBadgeTone } from './variants'
import { saveBadgeVariants } from './variants'

defineProps<{ tone?: SaveBadgeTone }>()
</script>

<template>
  <span :class="saveBadgeVariants({ tone })"><slot /></span>
</template>
