<script setup lang="ts">
import {
  ChevronLeft,
  Ellipsis,
  EllipsisVertical,
  Heart,
  Play,
  Settings,
  Trash2,
  Undo2,
} from '@lucide/vue'
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
    <KitState label="Primary · Hero (phone, full width)">
      <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Hero" class="w-field">
        <Play :size="18" fill="currentColor" :stroke-width="0" aria-hidden="true" />
        Resume
      </Button>
    </KitState>
    <KitState label="Soft · Tall">
      <div class="flex items-center gap-8">
        <Button :size="ButtonSize.Tall">
          <Heart :size="16" :stroke-width="1.75" aria-hidden="true" />
          Add to favourites
        </Button>
        <Button :size="ButtonSize.Tall" aria-label="More">
          <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="Soft · Touch (phone)">
      <Button :size="ButtonSize.Touch">Show the backups</Button>
    </KitState>
    <KitState label="Plain · IconTouch (phone)">
      <div class="flex items-center gap-8">
        <Button :variant="ButtonVariant.Plain" :size="ButtonSize.IconTouch" aria-label="Back">
          <ChevronLeft :size="24" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <Button :variant="ButtonVariant.Plain" :size="ButtonSize.IconTouch" aria-label="More">
          <EllipsisVertical :size="22" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="Text · Bare">
      <Button :variant="ButtonVariant.Text" :size="ButtonSize.Bare">
        <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
        Library
      </Button>
    </KitState>
    <KitState label="Link · Bare">
      <Button :variant="ButtonVariant.Link" :size="ButtonSize.Bare">Open every setting</Button>
    </KitState>
    <KitState label="Link · disabled">
      <Button :variant="ButtonVariant.Link" :size="ButtonSize.Bare" disabled>
        Open every setting
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
  </KitSection>
</template>
