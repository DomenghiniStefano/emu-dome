<script setup lang="ts">
import {
  Bell,
  ChevronLeft,
  ChevronRight,
  Ellipsis,
  EllipsisVertical,
  Heart,
  Library,
  Play,
  RefreshCw,
  Save,
  Search,
  Settings,
  Star,
  Trash2,
  Undo2,
} from '@lucide/vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { ACCENT_SWATCHES } from '@/lib/accent'
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
      <div class="flex items-center gap-8">
        <Button :variant="ButtonVariant.Soft" :fade="ButtonFade.Dim">Choose a folder</Button>
        <Button :variant="ButtonVariant.Soft" :fade="ButtonFade.Dim" disabled>Sign in</Button>
      </div>
    </KitState>
    <KitState label="Tall (40, standard label)">
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
    <KitState label="Pad: shoulder, menu, face, system">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Pad" :size="ButtonSize.PadShoulder">L</Button>
        <Button :variant="ButtonVariant.Pad" :size="ButtonSize.PadMenu" aria-label="Menu">
          <Settings :size="20" :stroke-width="2" aria-hidden="true" />
        </Button>
        <Button :variant="ButtonVariant.Pad" :size="ButtonSize.PadFace">A</Button>
        <Button :variant="ButtonVariant.Pad" :size="ButtonSize.PadSystem">START</Button>
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
          <span class="text-rail font-medium">Library</span>
        </Button>
        <Button :variant="ButtonVariant.Rail" :size="ButtonSize.Rail">
          <Save :size="20" :stroke-width="1.75" aria-hidden="true" />
          <span class="text-rail font-medium">Saves</span>
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
    <KitState label="Quick menu · Menu, Menu disabled, MenuCompact">
      <div class="flex items-center gap-8">
        <Button :size="ButtonSize.Menu">Save to slot 1</Button>
        <Button :size="ButtonSize.Menu" :fade="ButtonFade.Dim" disabled>Slot 2 is empty</Button>
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
    <KitState label="Pick · MenuCompact: one on, one disabled (fade Dim), one on the phone">
      <div class="flex items-center gap-8">
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
      <div class="flex items-center gap-8">
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
      <div class="flex items-center gap-8">
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.MenuMain">
          <Play :size="18" fill="currentColor" :stroke-width="0" aria-hidden="true" />
          Resume game
        </Button>
        <div data-density="touch" class="grid w-field text-lead">
          <Button :variant="ButtonVariant.Primary" :size="ButtonSize.MenuMain">Resume</Button>
        </div>
      </div>
    </KitState>
    <KitState label="Ghost · Back, Step, Step disabled, Step on the phone, IconTouch">
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
      <div class="flex items-center gap-8">
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
      <div class="flex w-sidebar flex-col gap-2">
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
      <div class="flex w-page flex-col rounded-10 border border-line bg-surface">
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
      <div class="flex items-center gap-8 bg-glass-thin p-6">
        <Button :variant="ButtonVariant.Field" :size="ButtonSize.Compact" class="w-topbar-search">
          <Search :size="16" :stroke-width="1.75" aria-hidden="true" />
          <span class="flex-1 text-left">Search games, actions and settings</span>
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
      <div class="flex w-panel flex-col glass-thick p-8">
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
      <div class="flex w-notifications flex-col divide-y divide-line glass-thick p-12">
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
  </KitSection>
</template>
