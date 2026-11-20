<script setup lang="ts">
// A blocking dialog of the game flow (G02, E04, E11, E12, the resume question), on the Dialog
// primitive: glass panel over the scrim, its own nav scope while shown (the `data-nav-initial`
// action takes the focus; B is `back`). Board rule: one action in accent at most, the safe exit
// always there. The parent shows it with `v-if`; it is open while mounted.
//
// A click on the scrim does nothing. In the game flow Escape does nothing either: B (or the safe
// exit's button) is the way out, and Escape keeps its meaning in App.vue (the quick menu in game).
// A dialog outside the game (Saves) sets `escapeCloses`: Escape is its safe exit, like B.
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
  DialogTone,
} from '@/components/ui/dialog'

const props = withDefaults(
  defineProps<{ title: string; tone?: DialogTone; escapeCloses?: boolean }>(),
  { tone: DialogTone.Default, escapeCloses: false },
)
const emit = defineEmits<{ back: [] }>()

const keepOpen = (e: Event) => e.preventDefault()
const onEscape = (e: Event) => (props.escapeCloses ? undefined : keepOpen(e))
</script>

<template>
  <Dialog :open="true" @update:open="open => open || emit('back')">
    <DialogContent role="alertdialog" @escape-key-down="onEscape" @interact-outside="keepOpen">
      <DialogTitle :tone="tone">{{ title }}</DialogTitle>
      <DialogDescription as="div" class="flex flex-col gap-6">
        <slot />
      </DialogDescription>
      <DialogFooter><slot name="actions" /></DialogFooter>
    </DialogContent>
  </Dialog>
</template>
