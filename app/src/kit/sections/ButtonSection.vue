<script setup lang="ts">
import { ChevronLeft, ChevronRight, Play, RefreshCw, Settings, Trash2, Undo2 } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'
</script>

<template>
  <KitSection title="Button">
    <KitState label="Primary · Large">
      <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Large" data-nav-initial>
        <Play :size="16" :stroke-width="1.75" aria-hidden="true" />
        Resume
      </Button>
    </KitState>
    <KitState label="Primary · Default">
      <Button :variant="ButtonVariant.Primary">Play</Button>
    </KitState>
    <KitState label="Primary · disabled">
      <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Large" disabled>Play</Button>
    </KitState>
    <KitState label="Soft">
      <Button>Choose a folder</Button>
    </KitState>
    <KitState label="Soft · disabled">
      <Button disabled>Choose a folder</Button>
    </KitState>
    <KitState label="Ghost · Small">
      <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Small">
        <Undo2 :size="14" :stroke-width="1.75" aria-hidden="true" />
        Reset
      </Button>
    </KitState>
    <KitState label="Ghost accent · Small">
      <Button :variant="ButtonVariant.GhostAccent" :size="ButtonSize.Small">Done</Button>
    </KitState>
    <KitState label="Ghost · disabled">
      <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Small" disabled>Reset</Button>
    </KitState>
    <KitState label="Danger">
      <Button :variant="ButtonVariant.Danger">
        <Trash2 :size="16" :stroke-width="1.75" aria-hidden="true" />
        Delete the save
      </Button>
    </KitState>
    <KitState label="Icon · IconSmall">
      <div class="flex items-center gap-8">
        <Button :size="ButtonSize.Icon" aria-label="Settings">
          <Settings :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.IconSmall" aria-label="Settings">
          <Settings :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="Quick menu · Menu, Menu disabled, MenuCompact">
      <div class="flex items-center gap-8">
        <Button :size="ButtonSize.Menu">Save to slot 1</Button>
        <Button :size="ButtonSize.Menu" disabled>Slot 2 is empty</Button>
        <Button :size="ButtonSize.MenuCompact">Undo changes</Button>
      </div>
    </KitState>
    <KitState label="Strong · MenuCompact · DangerOutline · Menu">
      <div class="flex items-center gap-8">
        <Button :variant="ButtonVariant.Strong" :size="ButtonSize.MenuCompact">
          Save for this game
        </Button>
        <Button :variant="ButtonVariant.DangerOutline" :size="ButtonSize.Menu">
          <RefreshCw :size="18" :stroke-width="1.75" aria-hidden="true" />
          Restart
        </Button>
      </div>
    </KitState>
    <KitState label="Touch · MenuTouch, MenuCompactTouch, MenuPairTouch (Strong)">
      <div class="flex items-center gap-8">
        <Button :size="ButtonSize.MenuTouch">Load</Button>
        <Button :size="ButtonSize.MenuCompactTouch">Undo load</Button>
        <Button :variant="ButtonVariant.Strong" :size="ButtonSize.MenuPairTouch">
          Save for this game
        </Button>
      </div>
    </KitState>
    <KitState label="Primary · MenuMain, MenuMainTouch">
      <div class="flex items-center gap-8">
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.MenuMain">
          <Play :size="18" fill="currentColor" :stroke-width="0" aria-hidden="true" />
          Resume game
        </Button>
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.MenuMainTouch">Resume</Button>
      </div>
    </KitState>
    <KitState label="Ghost · Back, Step, Step disabled, StepTouch, IconTouch">
      <div class="flex items-center gap-8">
        <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Back">
          <ChevronLeft :size="18" :stroke-width="1.75" aria-hidden="true" />
          Quick menu
        </Button>
        <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Step" aria-label="Next">
          <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Step" disabled aria-label="Next">
          <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.StepTouch" aria-label="Next">
          <ChevronRight :size="18" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.IconTouch" aria-label="Back">
          <ChevronLeft :size="22" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="Link · Text, TextTouch">
      <div class="flex items-center gap-8">
        <Button :variant="ButtonVariant.Link" :size="ButtonSize.Text">More slots and saves</Button>
        <Button :variant="ButtonVariant.Link" :size="ButtonSize.TextTouch">More slots</Button>
      </div>
    </KitState>
  </KitSection>
</template>
