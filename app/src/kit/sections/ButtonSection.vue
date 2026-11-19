<script setup lang="ts">
import {
  ChevronLeft,
  Ellipsis,
  Library,
  Play,
  Save,
  Settings,
  Star,
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
    <KitState label="Medium (dialog actions)">
      <div class="flex items-center gap-8">
        <Button :size="ButtonSize.Medium">Not now</Button>
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium">Resume</Button>
      </div>
    </KitState>
    <KitState label="RaisedAccent · Toast (a toast's action)">
      <Button :variant="ButtonVariant.RaisedAccent" :size="ButtonSize.Toast">Undo</Button>
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
    <KitState label="SoftAccent · TileAction, Soft · TileAction (a tile's actions)">
      <div class="flex w-field gap-6">
        <Button
          :variant="ButtonVariant.SoftAccent"
          :size="ButtonSize.TileAction"
          class="min-w-0 flex-1"
        >
          Play from here
        </Button>
        <Button :size="ButtonSize.TileAction" aria-label="More">
          <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="SoftAccent · TileAction · disabled">
      <Button :variant="ButtonVariant.SoftAccent" :size="ButtonSize.TileAction" disabled>
        Play from here
      </Button>
    </KitState>
    <KitState label="Back · TextLine">
      <Button :variant="ButtonVariant.Back" :size="ButtonSize.TextLine">
        <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
        Controls
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
    <KitState label="Quiet">
      <Button :variant="ButtonVariant.Quiet">Sort by: last played</Button>
    </KitState>
    <KitState label="Primary and Soft · Compact">
      <div class="flex items-center gap-8">
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Compact">Resume</Button>
        <Button :size="ButtonSize.Compact">Saves</Button>
      </div>
    </KitState>
    <KitState label="Primary and Soft · Medium">
      <div class="flex items-center gap-8">
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium">Choose a folder</Button>
        <Button :size="ButtonSize.Medium">Open a file</Button>
      </div>
    </KitState>
    <KitState label="Rail · current and not">
      <div class="flex flex-col gap-4 bg-glass-thin p-6">
        <Button :variant="ButtonVariant.Rail" :size="ButtonSize.Rail" aria-current="page">
          <Library :size="20" :stroke-width="1.75" aria-hidden="true" />
          <span class="text-[10px]/[12px] font-medium">Library</span>
        </Button>
        <Button :variant="ButtonVariant.Rail" :size="ButtonSize.Rail">
          <Save :size="20" :stroke-width="1.75" aria-hidden="true" />
          <span class="text-[10px]/[12px] font-medium">Saves</span>
        </Button>
      </div>
    </KitState>
    <KitState label="Inverse · CoverAction, Veil · CoverActionIcon">
      <div class="flex w-field gap-6 rounded-10 bg-cover-2 p-8">
        <Button :variant="ButtonVariant.Inverse" :size="ButtonSize.CoverAction" class="flex-1">
          <Play :size="14" fill="currentColor" :stroke-width="0" aria-hidden="true" />
          Play
        </Button>
        <Button :variant="ButtonVariant.Veil" :size="ButtonSize.CoverActionIcon" aria-label="More">
          <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="Menu · MenuItem, Menu · MenuRow (selected and not)">
      <div class="glass-thick flex w-field flex-col gap-2 p-6">
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
          <Star :size="16" :stroke-width="1.75" aria-hidden="true" />
          Add to favourites
        </Button>
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" data-selected>
          Last played
        </Button>
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow">Name</Button>
      </div>
    </KitState>
    <KitState label="Bare · Area (a click area over a picture)">
      <div class="relative size-48 rounded-10 bg-cover-1">
        <Button
          :variant="ButtonVariant.Bare"
          :size="ButtonSize.Area"
          class="absolute inset-0"
          aria-label="Open"
        ></Button>
      </div>
    </KitState>
  </KitSection>
</template>
