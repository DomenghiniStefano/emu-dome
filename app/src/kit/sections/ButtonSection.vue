<script setup lang="ts">
import { ChevronRight, Play, Settings, Trash2, Undo2 } from '@lucide/vue'
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
    <KitState label="Row buttons (settings rows): not faded when disabled, the row fades instead">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.RowSoft">Choose an image…</Button>
        <Button :variant="ButtonVariant.RowSoft" disabled>Export…</Button>
        <Button :variant="ButtonVariant.RowGhost" :size="ButtonSize.Small">
          <Undo2 :size="14" :stroke-width="1.75" aria-hidden="true" />
          Reset
        </Button>
        <Button :variant="ButtonVariant.RowGhostAccent" :size="ButtonSize.Small" disabled>
          Edit the list
        </Button>
      </div>
    </KitState>
    <KitState label="NavItem · NavItem: a side list, the current page marked">
      <div class="flex w-sidebar flex-col gap-2">
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
          v-for="(c, i) in ['#7152f7', '#2f7ae5', '#1f9d8a']"
          :key="c"
          :variant="ButtonVariant.Swatch"
          :size="ButtonSize.IconSmall"
          role="radio"
          :aria-checked="i === 0"
          :aria-label="c"
          :style="{ background: c }"
          :disabled="i === 2"
        />
      </div>
    </KitState>
  </KitSection>
</template>
