<script setup lang="ts">
// One item of a RadioGroup: Reka's radio behaviour (role, `aria-checked`, `data-state`, roving
// focus) put on its only child, the screen's own primitive (a Card or a Button), which keeps its
// look. Enter is left to nav.ts (`confirm` clicks it, which chooses it).
import type { RadioGroupItemProps } from 'reka-ui'
import { RadioGroupItem } from 'reka-ui'

const props = withDefaults(defineProps<RadioGroupItemProps>(), { asChild: true })
</script>

<template>
  <RadioGroupItem v-bind="props">
    <slot />
  </RadioGroupItem>
</template>
