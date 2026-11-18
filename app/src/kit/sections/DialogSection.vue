<script setup lang="ts">
import { ref } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const open = ref(false)
</script>

<template>
  <KitSection title="Dialog">
    <KitState label="Closed: the trigger (open it to see the panel)">
      <Dialog v-model:open="open">
        <DialogTrigger as-child>
          <Button>Restart the game</Button>
        </DialogTrigger>
        <DialogContent>
          <DialogTitle>Restart the game?</DialogTitle>
          <DialogDescription>What you did since the last save is lost.</DialogDescription>
          <DialogFooter>
            <DialogClose as-child>
              <Button data-nav-initial>Cancel</Button>
            </DialogClose>
            <DialogClose as-child>
              <Button :variant="ButtonVariant.Primary">Restart</Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </KitState>
    <KitState label="Footer buttons, as in FlowDialog">
      <div class="flex gap-8">
        <Button>Back to the library</Button>
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Default">Try again</Button>
      </div>
    </KitState>
  </KitSection>
</template>
