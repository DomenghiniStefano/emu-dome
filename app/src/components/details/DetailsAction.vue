<script setup lang="ts">
// An action in a card of the game details' tabs («Mostra i backup», «Cambia…»): the standard
// Soft button, at the touch size on the phone. Every attribute and listener goes to the button.
import { Button, ButtonSize } from '@/components/ui/button'

defineProps<{ phone: boolean }>()
</script>

<template>
  <Button :size="phone ? ButtonSize.Touch : ButtonSize.Default">
    <slot />
  </Button>
</template>
