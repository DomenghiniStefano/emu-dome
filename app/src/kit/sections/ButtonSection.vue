<script setup lang="ts">
import { Ellipsis, Play, Settings, Trash2, Undo2 } from '@lucide/vue'
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
    <KitState label="Soft · Medium (a dialog's actions)">
      <Button :size="ButtonSize.Medium">Export…</Button>
    </KitState>
    <KitState label="Primary · Medium">
      <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium">Play from here</Button>
    </KitState>
    <KitState label="Danger · Medium">
      <Button :variant="ButtonVariant.Danger" :size="ButtonSize.Medium">Delete slot 1</Button>
    </KitState>
    <KitState label="SoftAccent · Compact, Soft · Compact (a tile's actions)">
      <div class="flex w-field gap-6">
        <Button
          :variant="ButtonVariant.SoftAccent"
          :size="ButtonSize.Compact"
          class="min-w-0 flex-1"
        >
          Play from here
        </Button>
        <Button :size="ButtonSize.Compact" aria-label="More">
          <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="SoftAccent · Compact · disabled">
      <Button :variant="ButtonVariant.SoftAccent" :size="ButtonSize.Compact" disabled>
        Play from here
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
