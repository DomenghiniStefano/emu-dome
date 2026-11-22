<script setup lang="ts">
// An action in a card of the game details' tabs («Mostra i backup», «Cambia…»): the standard
// Soft button, at the touch size on the phone (ButtonSize.CardAction). Every attribute and listener
// goes to the button, its `icon` too (#104: each action draws its icon before the words).
import { Button, ButtonSize } from '@/components/ui/button'
</script>

<template>
  <Button :size="ButtonSize.CardAction">
    <slot />
  </Button>
</template>
