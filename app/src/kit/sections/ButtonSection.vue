<script setup lang="ts">
import { Play, Settings, Trash2, Undo2 } from '@lucide/vue'
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
    <KitState label="Medium (dialog actions)">
      <div class="flex items-center gap-8">
        <Button :size="ButtonSize.Medium">Not now</Button>
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium">Resume</Button>
      </div>
    </KitState>
    <KitState label="Soft accent · Toast (a toast's action)">
      <Button :variant="ButtonVariant.SoftAccent" :size="ButtonSize.Toast">Undo</Button>
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
    <KitState label="Ghost body · Small (a row's quiet action)">
      <Button :variant="ButtonVariant.GhostBody" :size="ButtonSize.Small">Remove</Button>
    </KitState>
    <KitState label="Soft row · Default, and disabled">
      <div class="flex items-center gap-8">
        <Button :variant="ButtonVariant.SoftRow">Choose a folder</Button>
        <Button :variant="ButtonVariant.SoftRow" disabled>Sign in</Button>
      </div>
    </KitState>
    <KitState label="Tall (40, standard label)">
      <Button :size="ButtonSize.Tall">Add a folder</Button>
    </KitState>
    <KitState label="Lead sizes (phone first run): Small, Default, Tall, Footer, Main">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.GhostBody" :size="ButtonSize.SmallLead">Remove</Button>
        <Button :variant="ButtonVariant.SoftRow" :size="ButtonSize.DefaultLead">Choose</Button>
        <Button :size="ButtonSize.TallLead">Add a folder</Button>
        <Button :size="ButtonSize.FooterLead">Back</Button>
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.MainLead">Continue</Button>
      </div>
    </KitState>
    <KitState label="Touch: shoulder, menu, face, system">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Touch" :size="ButtonSize.TouchShoulder">L</Button>
        <Button :variant="ButtonVariant.Touch" :size="ButtonSize.TouchMenu" aria-label="Menu">
          <Settings :size="20" :stroke-width="2" aria-hidden="true" />
        </Button>
        <Button :variant="ButtonVariant.Touch" :size="ButtonSize.TouchFace">A</Button>
        <Button :variant="ButtonVariant.Touch" :size="ButtonSize.TouchSystem">START</Button>
      </div>
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
  </KitSection>
</template>
