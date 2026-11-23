<script setup lang="ts">
import {
  Bell,
  ChevronLeft,
  ChevronRight,
  Ellipsis,
  EllipsisVertical,
  FastForward,
  Heart,
  Library,
  Play,
  Power,
  RefreshCw,
  Save,
  Search,
  Settings,
  Star,
  RotateCcw,
  Trash2,
  Undo2,
  VolumeX,
} from '@lucide/vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Card, CardVariant } from '@/design-system/components/card'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus } from '@/design-system/kit/review'
import { ACCENT_SWATCHES } from '@/design-system/lib/accent'
import PadShape from '@/components/touch/PadShape.vue'

/** The touch controls' two styles (`touch.tskin`). */
const PAD_STYLES = [
  { name: 'PadSmoke («Fumé»)', variant: ButtonVariant.PadSmoke },
  { name: 'PadOutline («Contorno»)', variant: ButtonVariant.PadOutline },
] as const
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
      <div class="flex flex-wrap items-center gap-8">
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
    <KitState label="Primary · Large on the phone (touch density, full width)">
      <div data-density="touch" class="flex">
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Large" class="w-field">
          <Play
            :size="16"
            class="touch:size-18"
            fill="currentColor"
            :stroke-width="0"
            aria-hidden="true"
          />
          Resume
        </Button>
      </div>
    </KitState>
    <KitState label="Primary · Large · aria-disabled (takes a press to say why)">
      <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Large" aria-disabled="true">
        Resume
      </Button>
    </KitState>
    <KitState label="Soft · Tall">
      <div class="flex flex-wrap items-center gap-8">
        <Button :size="ButtonSize.Tall">
          <Heart :size="16" :stroke-width="1.75" aria-hidden="true" />
          Add to favourites
        </Button>
        <Button :size="ButtonSize.Tall" aria-label="More">
          <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="SoftAccent · TileAction · disabled">
      <Button :variant="ButtonVariant.SoftAccent" :size="ButtonSize.TileAction" disabled>
        Play from here
      </Button>
    </KitState>
    <KitState label="Ghost · TextLine">
      <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.TextLine">
        <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
        Controls
      </Button>
    </KitState>
    <KitState label="GhostRegular · Small (a row's quiet action)">
      <Button :variant="ButtonVariant.GhostRegular" :size="ButtonSize.Small">Remove</Button>
    </KitState>
    <KitState label="Soft · fade Dim, and disabled (half faded)">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Soft" :fade="ButtonFade.Dim">Choose a folder</Button>
        <Button :variant="ButtonVariant.Soft" :fade="ButtonFade.Dim" disabled>Sign in</Button>
      </div>
    </KitState>
    <KitState label="Tall (control-lg, standard label)">
      <Button :size="ButtonSize.Tall">Add a folder</Button>
    </KitState>
    <KitState label="Footer: desktop, and on the phone">
      <div class="flex flex-wrap items-center gap-8">
        <Button :size="ButtonSize.Footer">Back</Button>
        <div data-density="touch" class="flex">
          <Button :size="ButtonSize.Footer">Back</Button>
        </div>
      </div>
    </KitState>
    <KitState label="On the phone (touch density): Small, Default, Tall, Footer, Large">
      <div data-density="touch" class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.GhostRegular" :size="ButtonSize.Small">Remove</Button>
        <Button :fade="ButtonFade.Dim">Choose</Button>
        <Button :size="ButtonSize.Tall">Add a folder</Button>
        <Button :size="ButtonSize.Footer">Back</Button>
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Large">Continue</Button>
      </div>
    </KitState>
    <KitState
      v-for="pad in PAD_STYLES"
      :key="pad.variant"
      :label="`${pad.name}: shoulder, menu, face, system, a held face (over a game's colours, at the style's fade)`"
    >
      <div class="flex flex-wrap items-center gap-8 bg-cover-2 p-8">
        <div class="flex flex-wrap items-center gap-8 opacity-pad">
          <Button :variant="pad.variant" :size="ButtonSize.PadShoulder">
            <PadShape kind="shoulderL" /><span class="relative">L</span>
          </Button>
          <Button :variant="pad.variant" :size="ButtonSize.PadMenu" aria-label="Menu">
            <PadShape kind="menu" />
          </Button>
          <Button :variant="pad.variant" :size="ButtonSize.PadFace">
            <PadShape kind="face" /><span class="relative">A</span>
          </Button>
          <Button :variant="pad.variant" :size="ButtonSize.PadSystem">
            <PadShape kind="pill" /><span class="relative">START</span>
          </Button>
          <Button :variant="pad.variant" :size="ButtonSize.PadFace" aria-pressed="true">
            <PadShape kind="face" /><span class="relative">B</span>
          </Button>
          <Button :variant="pad.variant" :size="ButtonSize.PadShoulder">
            <PadShape kind="shoulderR" /><span class="relative">R</span>
          </Button>
        </div>
      </div>
    </KitState>
    <KitState label="Tool and ToolIcon (the C06 editor's toolbar): desktop, and on the phone">
      <div class="flex flex-wrap items-center gap-8">
        <Button :size="ButtonSize.ToolIcon" aria-label="Undo">
          <Settings :size="18" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <Button :size="ButtonSize.Tool">Size</Button>
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Tool">Save</Button>
        <div data-density="touch" class="flex gap-8 text-lead">
          <Button :size="ButtonSize.ToolIcon" aria-label="Undo">
            <Settings :size="18" :stroke-width="1.75" aria-hidden="true" />
          </Button>
          <Button :size="ButtonSize.Tool">Size</Button>
          <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Tool">Save</Button>
        </div>
      </div>
    </KitState>
    <KitState label="Soft · CardAction: desktop, and on the phone">
      <div class="flex flex-wrap items-center gap-8">
        <Button :size="ButtonSize.CardAction">Show the backups</Button>
        <div data-density="touch" class="flex text-lead">
          <Button :size="ButtonSize.CardAction">Show the backups</Button>
        </div>
      </div>
    </KitState>
    <KitState label="Plain · IconTouch (phone)">
      <div class="flex flex-wrap items-center gap-8">
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
    <KitState label="Link · Bare · disabled (faint, not faded), and on the phone">
      <Button
        :variant="ButtonVariant.Link"
        :size="ButtonSize.Bare"
        :fade="ButtonFade.None"
        disabled
      >
        Open every setting
      </Button>
      <div data-density="touch" class="flex">
        <Button :variant="ButtonVariant.Link" :size="ButtonSize.Bare">Open every setting</Button>
      </div>
    </KitState>
    <KitState label="Link · Bare · disabled with the normal fade">
      <Button :variant="ButtonVariant.Link" :size="ButtonSize.Bare" disabled>
        Open every setting
      </Button>
    </KitState>
    <KitState label="Icon · IconSmall">
      <div class="flex flex-wrap items-center gap-8">
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
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Compact">Resume</Button>
        <Button :size="ButtonSize.Compact">Saves</Button>
      </div>
    </KitState>
    <KitState label="Primary and Soft · Medium">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium">Choose a folder</Button>
        <Button :size="ButtonSize.Medium">Open a file</Button>
      </div>
    </KitState>
    <KitState label="Rail · current and not">
      <div class="flex flex-col gap-4 bg-glass-thin p-6">
        <Button :variant="ButtonVariant.Rail" :size="ButtonSize.Rail" aria-current="page">
          <Library :size="20" :stroke-width="1.75" aria-hidden="true" />
          <span class="text-rail font-medium">Library</span>
        </Button>
        <Button :variant="ButtonVariant.Rail" :size="ButtonSize.Rail">
          <Save :size="20" :stroke-width="1.75" aria-hidden="true" />
          <span class="text-rail font-medium">Saves</span>
        </Button>
      </div>
    </KitState>
    <KitState label="PhoneTab · current and not (the phone's tab bar, #29)">
      <div class="flex w-full border-t border-line glass-thick px-4 pt-4 pb-4">
        <Button :variant="ButtonVariant.PhoneTab" :size="ButtonSize.PhoneTab" aria-current="page">
          <Library :size="24" :stroke-width="1.75" aria-hidden="true" />
          <span class="text-fine font-medium">Library</span>
        </Button>
        <Button :variant="ButtonVariant.PhoneTab" :size="ButtonSize.PhoneTab">
          <Save :size="24" :stroke-width="1.75" aria-hidden="true" />
          <span class="text-fine font-medium">Saves</span>
        </Button>
      </div>
    </KitState>
    <KitState label="TvTab · current and not (TV mode's section tabs)">
      <div class="flex items-center gap-24">
        <Button :variant="ButtonVariant.TvTab" :size="ButtonSize.TvTab" aria-current="page">
          Library
        </Button>
        <Button :variant="ButtonVariant.TvTab" :size="ButtonSize.TvTab">Saves</Button>
      </div>
    </KitState>
    <KitState label="Primary · Tv, Soft · Tv (TV mode's hero actions)">
      <div class="flex flex-wrap items-center gap-16">
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Tv">Resume</Button>
        <Button :size="ButtonSize.Tv">Game details</Button>
      </div>
    </KitState>
    <KitState label="NavItem · TvNavItem, current and not (TV mode's settings pages)">
      <div class="flex w-panel max-w-full flex-col gap-8">
        <Button :variant="ButtonVariant.NavItem" :size="ButtonSize.TvNavItem" aria-current="page">
          Video
        </Button>
        <Button :variant="ButtonVariant.NavItem" :size="ButtonSize.TvNavItem">Audio</Button>
      </div>
    </KitState>
    <KitState label="TvRow · TvRow (TV mode's settings rows and sheet options)">
      <div class="flex w-panel max-w-full flex-col gap-8">
        <Button :variant="ButtonVariant.TvRow" :size="ButtonSize.TvRow">
          <span>Show FPS</span>
          <span class="text-ink-muted">Off</span>
        </Button>
        <Button :variant="ButtonVariant.TvRow" :size="ButtonSize.TvRow" disabled>
          <span>Theme</span>
          <span class="font-semibold">Dark</span>
        </Button>
      </div>
    </KitState>
    <KitState label="Inverse · CoverAction, Veil · CoverActionIcon">
      <div class="flex w-field gap-6 rounded-card bg-cover-2 p-8">
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
      <div class="relative size-48 rounded-card bg-cover-1">
        <Button
          :variant="ButtonVariant.Bare"
          :size="ButtonSize.Area"
          class="absolute inset-0"
          aria-label="Open"
        ></Button>
      </div>
    </KitState>
    <KitState label="Quick menu · Menu, Menu disabled, MenuCompact">
      <div class="flex flex-wrap items-center gap-8">
        <Button :size="ButtonSize.Menu">Save to slot 1</Button>
        <Button :size="ButtonSize.Menu" :fade="ButtonFade.Dim" disabled>Slot 2 is empty</Button>
        <Button :size="ButtonSize.MenuCompact">Undo changes</Button>
      </div>
    </KitState>
    <KitState label="Strong · MenuCompact · DangerOutline · Menu">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Strong" :size="ButtonSize.MenuCompact">
          Save for this game
        </Button>
        <Button :variant="ButtonVariant.DangerOutline" :size="ButtonSize.Menu">
          <RefreshCw :size="18" :stroke-width="1.75" aria-hidden="true" />
          Restart
        </Button>
      </div>
    </KitState>
    <KitState label="Pick · MenuCompact: one on, one disabled (fade Dim), one on the phone">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Pick" :size="ButtonSize.MenuCompact" :aria-pressed="true">
          Sharp pixels
        </Button>
        <Button :variant="ButtonVariant.Pick" :size="ButtonSize.MenuCompact" :aria-pressed="false">
          Soft
        </Button>
        <Button
          :variant="ButtonVariant.Pick"
          :size="ButtonSize.MenuCompact"
          :aria-pressed="false"
          :fade="ButtonFade.Dim"
          disabled
        >
          CRT TV
        </Button>
        <div data-density="touch" class="flex">
          <Button
            :variant="ButtonVariant.Pick"
            :size="ButtonSize.MenuCompact"
            :aria-pressed="false"
          >
            LCD screen
          </Button>
        </div>
      </div>
    </KitState>
    <KitState label="MenuPair (Soft, Strong): desktop">
      <div class="flex flex-wrap items-center gap-8">
        <Button :size="ButtonSize.MenuPair">Undo</Button>
        <Button :variant="ButtonVariant.Strong" :size="ButtonSize.MenuPair">
          Save for this game
        </Button>
      </div>
    </KitState>
    <KitState label="On the phone · Menu, MenuCompact, MenuPair (Strong)">
      <div data-density="touch" class="flex items-center gap-8 text-lead">
        <Button :size="ButtonSize.Menu">Load</Button>
        <Button :size="ButtonSize.MenuCompact">Undo load</Button>
        <Button :variant="ButtonVariant.Strong" :size="ButtonSize.MenuPair">
          Save for this game
        </Button>
      </div>
    </KitState>
    <KitState label="Primary · MenuMain: desktop, and on the phone">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.MenuMain">
          <Play :size="18" fill="currentColor" :stroke-width="0" aria-hidden="true" />
          Resume game
        </Button>
        <div data-density="touch" class="grid w-field text-lead">
          <Button :variant="ButtonVariant.Primary" :size="ButtonSize.MenuMain">Resume</Button>
        </div>
      </div>
    </KitState>
    <KitState
      label="The phone quick menu's first level: QuickTile (Soft, Primary), QuickRow, Strip (on: aria-pressed), StripDanger"
    >
      <div data-density="touch" class="flex w-panel flex-col gap-8 text-lead">
        <div class="grid grid-cols-3 gap-8">
          <Button :variant="ButtonVariant.Soft" :size="ButtonSize.QuickTile" :icon="Save">
            <span class="flex flex-col">
              <span class="font-semibold">Quick save</span>
              <span class="text-fine text-ink-muted">Slot 1</span>
            </span>
          </Button>
          <Button :variant="ButtonVariant.Soft" :size="ButtonSize.QuickTile" :icon="RotateCcw">
            <span class="flex flex-col">
              <span class="font-semibold">Quick load</span>
              <span class="text-fine text-ink-muted">Slot 1 · 21:39</span>
            </span>
          </Button>
          <Button
            :variant="ButtonVariant.Primary"
            :size="ButtonSize.QuickTile"
            :icon="Play"
            icon-filled
          >
            Resume
          </Button>
        </div>
        <Button :variant="ButtonVariant.Soft" :size="ButtonSize.QuickRow" :icon="Save">
          Quick save
        </Button>
        <div class="flex gap-4">
          <Button :variant="ButtonVariant.Strip" :size="ButtonSize.Strip" :icon="FastForward">
            Speed up
          </Button>
          <Button
            :variant="ButtonVariant.Strip"
            :size="ButtonSize.Strip"
            :icon="VolumeX"
            aria-pressed="true"
          >
            Unmute
          </Button>
          <Button
            :variant="ButtonVariant.StripDanger"
            :size="ButtonSize.Strip"
            :icon="Power"
            aria-label="Close game"
          >
            Close
          </Button>
          <Button :variant="ButtonVariant.Strip" :size="ButtonSize.Strip" :icon="Ellipsis">
            More
          </Button>
        </div>
      </div>
    </KitState>
    <KitState label="Ghost · Back, Step, Step disabled, Step on the phone, IconTouch">
      <div class="flex flex-wrap items-center gap-8">
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
        <div data-density="touch" class="flex">
          <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Step" aria-label="Next">
            <ChevronRight
              :size="16"
              class="touch:size-18"
              :stroke-width="1.75"
              aria-hidden="true"
            />
          </Button>
        </div>
        <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.IconTouch" aria-label="Back">
          <ChevronLeft :size="22" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="Link · Text, Link · LabelLink: desktop, and LabelLink on the phone">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Link" :size="ButtonSize.Text">Undo</Button>
        <Button :variant="ButtonVariant.Link" :size="ButtonSize.LabelLink"
          >More slots and saves</Button
        >
        <div data-density="touch" class="flex text-lead">
          <Button :variant="ButtonVariant.Link" :size="ButtonSize.LabelLink">More slots</Button>
        </div>
      </div>
    </KitState>
    <KitState label="RaisedAccent · Toast on the phone">
      <div data-density="touch" class="flex">
        <Button :variant="ButtonVariant.RaisedAccent" :size="ButtonSize.Toast">Undo</Button>
      </div>
    </KitState>
    <KitState label="fade None (settings rows): not faded when disabled, the row fades instead">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Soft" :fade="ButtonFade.None">Choose an image…</Button>
        <Button :variant="ButtonVariant.Soft" :fade="ButtonFade.None" disabled>Export…</Button>
        <Button
          :variant="ButtonVariant.GhostRegular"
          :fade="ButtonFade.None"
          :size="ButtonSize.Small"
        >
          <Undo2 :size="14" :stroke-width="1.75" aria-hidden="true" />
          Reset
        </Button>
        <Button
          :variant="ButtonVariant.GhostAccent"
          :fade="ButtonFade.None"
          :size="ButtonSize.Small"
          disabled
        >
          Edit the list
        </Button>
      </div>
    </KitState>
    <KitState label="NavHead, NavItem · NavItem: a side list, the current page marked">
      <div class="flex w-sidebar max-w-full flex-col gap-2">
        <Button :variant="ButtonVariant.NavHead" :size="ButtonSize.NavItem">
          <span class="flex-1 text-left">Overview</span>
        </Button>
        <Button :variant="ButtonVariant.NavItem" :size="ButtonSize.NavItem" aria-current="page">
          <span class="flex-1 text-left">Video</span>
        </Button>
        <Button :variant="ButtonVariant.NavItem" :size="ButtonSize.NavItem">
          <span class="flex-1 text-left">Screen effects</span>
        </Button>
      </div>
    </KitState>
    <KitState label="LinkRow · LinkRow: rows of a card list that open a page">
      <div class="flex w-page max-w-full flex-col rounded-card border border-line bg-surface">
        <Button v-for="n in 2" :key="n" :variant="ButtonVariant.LinkRow" :size="ButtonSize.LinkRow">
          <span class="flex min-w-0 flex-1 flex-col">
            <span class="font-medium text-ink">Screen effect {{ n }}</span>
            <span class="text-caption">Screen effects</span>
          </span>
          <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="Swatch · IconSmall: the chosen one ringed, a disabled one">
      <div class="flex gap-10" role="radiogroup" aria-label="Accent">
        <Button
          v-for="(c, i) in ACCENT_SWATCHES.slice(0, 3)"
          :key="c"
          :variant="ButtonVariant.Swatch"
          :size="ButtonSize.IconSmall"
          :fade="ButtonFade.None"
          role="radio"
          :aria-checked="i === 0"
          :aria-label="c"
          :style="{ background: c }"
          :disabled="i === 2"
        />
      </div>
    </KitState>
    <KitState label="Field · Compact: the top bar's search; BarIcon · Icon: closed and open">
      <div class="flex max-w-full flex-wrap items-center gap-8 bg-glass-thin p-6">
        <Button
          :variant="ButtonVariant.Field"
          :size="ButtonSize.Compact"
          class="w-topbar-search max-w-full min-w-0 shrink"
        >
          <Search :size="16" :stroke-width="1.75" aria-hidden="true" />
          <span class="min-w-0 flex-1 truncate text-left">Search games, actions and settings</span>
        </Button>
        <Button :variant="ButtonVariant.BarIcon" :size="ButtonSize.Icon" aria-label="Notifications">
          <Bell :size="20" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <Button
          :variant="ButtonVariant.BarIcon"
          :size="ButtonSize.Icon"
          aria-label="Notifications"
          aria-expanded="true"
        >
          <Bell :size="20" :stroke-width="1.75" aria-hidden="true" />
        </Button>
      </div>
    </KitState>
    <KitState label="Result · Result: a search result, one disabled with its reason">
      <div class="flex w-panel max-w-full flex-col glass-thick p-8">
        <Button :variant="ButtonVariant.Result" :size="ButtonSize.Result">
          <Settings :size="20" :stroke-width="1.75" class="text-ink-muted" aria-hidden="true" />
          <span class="flex flex-col">
            <span class="font-medium">Vertical sync (V-Sync)</span>
            <span class="text-caption text-ink-muted">Settings · Video</span>
          </span>
          <span class="ml-auto text-caption text-ink-muted">On</span>
        </Button>
        <Button :variant="ButtonVariant.Result" :size="ButtonSize.Result" disabled>
          <Save :size="20" :stroke-width="1.75" class="text-ink-muted" aria-hidden="true" />
          <span class="flex flex-col">
            <span class="font-medium">Quick save to the current slot</span>
            <span class="text-caption text-ink-muted">Open a game to use it</span>
          </span>
        </Button>
      </div>
    </KitState>
    <KitState label="Result · Notice: a notification, unread and read">
      <div class="flex w-notifications max-w-full flex-col divide-y divide-line glass-thick p-12">
        <Button v-for="n in 2" :key="n" :variant="ButtonVariant.Result" :size="ButtonSize.Notice">
          <span class="mt-5 size-8 flex-none rounded-full bg-ink-faint"></span>
          <span class="flex min-w-0 flex-1 flex-col gap-2">
            <span :class="n === 1 ? 'font-semibold' : 'font-medium'">Controller connected</span>
            <span class="text-caption text-ink-muted">Xbox Wireless Controller</span>
          </span>
          <span class="flex-none text-caption text-ink-muted">5 minutes ago</span>
        </Button>
      </div>
    </KitState>
    <template #review>
      <KitIssue title="Un'etichetta lunga esce dal suo posto" :status="IssueStatus.Open">
        Button, Chip, la tab di Tabs e Badge restano su una riga e non si stringono
        (whitespace-nowrap, shrink-0): con una traduzione lunga in un posto stretto (un telefono,
        una colonna) il pulsante esce dal riquadro che lo contiene e fa scorrere la pagina di lato.
        Segmented e Select invece accorciano con «…». Qui sotto un riquadro largo 200.
        <template #current>
          <div class="flex w-field max-w-full border border-dashed border-line">
            <Button>Ripristina le impostazioni di tutti i giochi</Button>
          </div>
        </template>
        <template #proposed>
          <div class="flex w-field max-w-full border border-dashed border-line">
            <Button class="max-w-full min-w-0 shrink">
              <span class="min-w-0 truncate">Ripristina le impostazioni di tutti i giochi</span>
            </Button>
          </div>
        </template>
        <template #solution>
          Da decidere col proprietario: o il pulsante si accorcia con «…» dentro il suo posto
          (max-w-full nella base di button/variants.ts e l'etichetta in uno span truncate in
          Button.vue, lo stesso per Chip e la tab), o va a capo su due righe come CardAction; oppure
          resta così e ogni testo lungo si controlla nelle schermate.
        </template>
      </KitIssue>
      <KitIssue
        title="Sul telefono Small, Default e Tall restano sotto i 44"
        :status="IssueStatus.Proposed"
      >
        Con la densità touch Small resta alto 24, Default 28 e Tall 32: cambia solo la misura del
        testo. Il minimo al tocco del design system è 44 (il token touch), e Footer, Toast, Step,
        CardAction e Bare lo rispettano già. In più, su desktop Tall e Footer hanno le stesse
        classi: sono due nomi per lo stesso pulsante, che si separano solo sul telefono.
        <template #current>
          <div data-density="touch" class="flex flex-wrap items-center gap-8">
            <Button :variant="ButtonVariant.GhostRegular" :size="ButtonSize.Small">Rimuovi</Button>
            <Button>Scegli</Button>
            <Button :size="ButtonSize.Tall">Aggiungi una cartella</Button>
            <Button :size="ButtonSize.Footer">Indietro</Button>
          </div>
        </template>
        <template #proposed>
          <div data-density="touch" class="flex flex-wrap items-center gap-8">
            <Button
              :variant="ButtonVariant.GhostRegular"
              :size="ButtonSize.Small"
              class="touch:h-touch"
            >
              Rimuovi
            </Button>
            <Button class="touch:h-touch">Scegli</Button>
            <Button :size="ButtonSize.Tall" class="touch:h-48">Aggiungi una cartella</Button>
            <Button :size="ButtonSize.Footer">Indietro</Button>
          </div>
        </template>
        <template #solution>
          In button/variants.ts Small e Default prendono touch:h-touch (44), Tall l'altezza di
          Footer sul telefono (48); a quel punto Footer è Tall e si può togliere (4 usi da
          spostare).
        </template>
      </KitIssue>
      <KitIssue title="Tre opacità per «disabilitato»" :status="IssueStatus.Proposed">
        Un controllo che non si può usare sbiadisce a 0,4 (opacity-disabled: Button, DateTimeField,
        Input Block), a 0,5 (opacity-dim: ButtonFade.Dim, Card Tool e PictureChoice) o a 0,55
        (opacity-locked: Card NavRow e ListRow, SelectItem, le righe bloccate delle impostazioni).
        Nel menu rapido un NavRow a 0,55 sta vicino a pulsanti Dim a 0,5: stessa cosa, tre valori.
        <template #current>
          <div class="flex w-field flex-col gap-8">
            <Button disabled>Normal · 0,4</Button>
            <Button :fade="ButtonFade.Dim" disabled>Dim · 0,5</Button>
            <Card as="button" :variant="CardVariant.NavRow" disabled>NavRow · 0,55</Card>
          </div>
        </template>
        <template #proposed>
          <div class="flex w-field flex-col gap-8">
            <Button disabled>Normal · 0,4</Button>
            <Button disabled>Dim → Normal · 0,4</Button>
            <Card
              as="button"
              :variant="CardVariant.NavRow"
              class="disabled:opacity-disabled"
              disabled
            >
              NavRow · 0,4
            </Card>
          </div>
        </template>
        <template #solution>
          Un solo token per «non si può usare» in opacity.css (il valore lo sceglie il
          proprietario); dim e locked diventano lo stesso valore o spariscono, ButtonFade.Dim si
          fonde in Normal, e Card / SelectItem / le righe delle impostazioni leggono quel token.
        </template>
      </KitIssue>
      <KitIssue title="In Chiaro Soft si stacca appena dallo sfondo" :status="IssueStatus.Proposed">
        In Chiaro il fondo di Soft (surface-raised) contro lo sfondo della pagina ha un contrasto di
        1,03:1 e il bordo line (nero al 7%) di 1,14:1: il pulsante si riconosce solo dal testo. Lo
        stesso line disegna i bordi di Card, Chip, Segmented, Select e il binario dello Slider.
        Passa il Kit a Chiaro per vederlo.
        <template #current>
          <Button>Scegli una cartella</Button>
          <Button :variant="ButtonVariant.Danger">Elimina</Button>
        </template>
        <template #proposed>
          <Button class="border-line-strong">Scegli una cartella</Button>
          <Button :variant="ButtonVariant.Danger" class="border-line-strong">Elimina</Button>
        </template>
        <template #solution>
          O alzare line di Chiaro (colors.css, il blocco data-theme light) a circa il 15% di nero,
          che sistema tutti i bordi insieme; o, come nella proposta, il bordo line-strong su Soft e
          Danger in button/variants.ts (pesa di più anche in Scuro e Nero).
        </template>
      </KitIssue>
      <KitIssue title="Il focus cancella i segni fatti con ombra o contorno">
        L'anello di focus (styles.css, :root[data-nav] :focus) imposta outline e box-shadow fuori
        dai layer di Tailwind, quindi vince su ogni classe. Col focus la sezione corrente del Rail
        perde la sua barretta (shadow-rail), il campione scelto di Swatch perde l'anello che lo
        distingue dagli altri (diventa uguale a uno non scelto con il focus) e Primary perde il
        bagliore. Per vederlo: premi una freccia e porta il focus su «Library» nel Rail o sul primo
        campione.
        <template #solution>
          Lo stato non si disegna con box-shadow né outline: la barretta del Rail e l'anello del
          campione passano su un bordo o un pseudo-elemento (button/variants.ts, utilities.css);
          oppure l'anello di focus rinuncia al box-shadow e resta solo outline. Stesso problema
          sulla tab attiva (vedi Tabs).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
