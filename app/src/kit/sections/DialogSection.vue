<script setup lang="ts">
import { ref } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogDescriptionSize,
  DialogFooter,
  DialogSize,
  DialogTitle,
  DialogTitleSize,
  DialogTone,
  DialogTrigger,
} from '@/components/ui/dialog'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'

const open = ref(false)
const openDanger = ref(false)
const openTv = ref(false)
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
    <KitState label="Danger tone, Medium actions (FlowDialog)">
      <Dialog v-model:open="openDanger">
        <DialogTrigger as-child>
          <Button>The emulator stopped</Button>
        </DialogTrigger>
        <DialogContent>
          <DialogTitle :tone="DialogTone.Danger">The emulator stopped</DialogTitle>
          <DialogDescription>Your progress was saved 2 minutes ago.</DialogDescription>
          <DialogFooter>
            <DialogClose as-child>
              <Button :size="ButtonSize.Medium">Back to the library</Button>
            </DialogClose>
            <DialogClose as-child>
              <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium" data-nav-initial>
                Resume
              </Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </KitState>
    <KitState label="TvSheet, Tv title and description (TV mode's full-screen choice)">
      <Dialog v-model:open="openTv">
        <DialogTrigger as-child>
          <Button>Theme</Button>
        </DialogTrigger>
        <DialogContent :size="DialogSize.TvSheet">
          <div class="flex flex-col gap-12 p-32">
            <DialogTitle :size="DialogTitleSize.Tv">Theme</DialogTitle>
            <DialogDescription :size="DialogDescriptionSize.Tv">
              How dark the interface is.
            </DialogDescription>
            <DialogClose as-child>
              <Button :variant="ButtonVariant.TvRow" :size="ButtonSize.TvRow" data-nav-initial>
                Dark
              </Button>
            </DialogClose>
          </div>
        </DialogContent>
      </Dialog>
    </KitState>
  </KitSection>
</template>
