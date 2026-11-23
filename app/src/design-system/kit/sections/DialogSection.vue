<script setup lang="ts">
import { ref } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Card, CardDescription, CardTitle } from '@/design-system/components/card'
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
} from '@/design-system/components/dialog'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'

const open = ref(false)
const openDanger = ref(false)
const openWarning = ref(false)
const openWide = ref(false)
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
              <Button :size="ButtonSize.Medium" data-nav-initial>Cancel</Button>
            </DialogClose>
            <DialogClose as-child>
              <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium">Restart</Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </KitState>
    <KitState label="Footer buttons, as in FlowDialog (Medium, control-md)">
      <div class="flex gap-8">
        <Button :size="ButtonSize.Medium">Back to the library</Button>
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium">Try again</Button>
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
    <KitState label="Warning tone (E04 the save cannot be read, E12 slow)">
      <Dialog v-model:open="openWarning">
        <DialogTrigger as-child>
          <Button>The save cannot be read</Button>
        </DialogTrigger>
        <DialogContent>
          <DialogTitle :tone="DialogTone.Warning">The save cannot be read</DialogTitle>
          <DialogDescription>The game can start without it, from the beginning.</DialogDescription>
          <DialogFooter>
            <DialogClose as-child>
              <Button :size="ButtonSize.Medium" data-nav-initial>Back to the library</Button>
            </DialogClose>
            <DialogClose as-child>
              <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium">
                Play without it
              </Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </KitState>
    <KitState label="Wide, Heading title (L12 «Choose the cover»: the screen behind blurred)">
      <Dialog v-model:open="openWide">
        <DialogTrigger as-child>
          <Button>Choose the cover</Button>
        </DialogTrigger>
        <DialogContent :size="DialogSize.Wide">
          <DialogTitle :size="DialogTitleSize.Heading">Choose the cover</DialogTitle>
          <DialogDescription>Pick one, or add your own image.</DialogDescription>
          <div class="grid grid-cols-3 gap-10">
            <Card v-for="n in 3" :key="n" class="gap-4">
              <CardTitle>Choice {{ n }}</CardTitle>
              <CardDescription>A picture</CardDescription>
            </Card>
          </div>
          <DialogFooter>
            <DialogClose as-child>
              <Button :size="ButtonSize.Medium" data-nav-initial>Close</Button>
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
    <template #review>
      <KitIssue title="La taglia delle azioni la sceglie ogni dialogo">
        Tutti i dialoghi del flusso di gioco (CrashDialog, ResumeDialog, SaveBrokenDialog,
        SlowDialog, StartFailedDialog, ClosingAsk) passano a mano ButtonSize.Medium a ogni pulsante
        del piede: DialogFooter non dice niente. Il Kit stesso mostrava il primo dialogo con
        pulsanti da 28 invece che da 30. Un dialogo nuovo che lo dimentica ha azioni più basse degli
        altri.
        <template #solution>
          DialogFooter dà la taglia ai suoi pulsanti (un contesto letto da Button, come Tabs fa con
          le sue tab), o un ButtonSize.Dialog nominato una volta in dialog/; i chiamanti smettono di
          scrivere la taglia.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
