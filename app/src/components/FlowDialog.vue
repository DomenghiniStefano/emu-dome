<script setup lang="ts">
// A blocking dialog of the game flow (G02, E04, E11, E12, the resume question), on the Dialog
// primitive: glass panel over the scrim, its own nav scope while shown (the `data-nav-initial`
// action takes the focus; B is `back`). Board rule: one action in accent at most, the safe exit
// always there. The parent shows it with `v-if`; it is open while mounted.
//
// Escape and a click on the scrim do nothing here, as before the primitive: B (or the safe exit's
// button) is the way out, and Escape keeps its meaning in App.vue (the quick menu in game).
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
  DialogTone,
} from '@/components/ui/dialog'

withDefaults(defineProps<{ title: string; tone?: DialogTone }>(), { tone: DialogTone.Default })
const emit = defineEmits<{ back: [] }>()

const keepOpen = (e: Event) => e.preventDefault()
</script>

<template>
  <Dialog :open="true" @update:open="open => open || emit('back')">
    <DialogContent role="alertdialog" @escape-key-down="keepOpen" @interact-outside="keepOpen">
      <DialogTitle :tone="tone">{{ title }}</DialogTitle>
      <DialogDescription as="div" class="flex flex-col gap-6">
        <slot />
      </DialogDescription>
      <DialogFooter><slot name="actions" /></DialogFooter>
    </DialogContent>
  </Dialog>
</template>
