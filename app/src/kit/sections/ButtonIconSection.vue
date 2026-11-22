<script setup lang="ts">
// Kit (Trello #104): Button's own icon, `icon` before the words and `icon` + `label` alone, at the
// sizes the app uses, on the desktop and on the phone (touch density). The review board of the
// text-only buttons was decided by the owner (2026-11-22) and carried by every screen, so it is
// gone; the rule is in docs/frontend-conventions.md («Icons on buttons»).
import { Check, FolderPlus, Pin, Play, Save, Share, Trash2, Undo2, X } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
</script>

<template>
  <KitSection title="Button · icona">
    <KitState label="icon + text · Primary Large (iconFilled: the app's play triangle)">
      <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Large" :icon="Play" icon-filled>
        Riprendi
      </Button>
    </KitState>
    <KitState label="icon + text · Medium (a dialog's pair)">
      <div class="flex flex-wrap items-center gap-8">
        <Button :size="ButtonSize.Medium" :icon="X">Annulla</Button>
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium" :icon="Check">
          Usa questa copertina
        </Button>
      </div>
    </KitState>
    <KitState label="icon + text · Default, Tall, Small">
      <div class="flex flex-wrap items-center gap-8">
        <Button :icon="FolderPlus">Aggiungi una cartella…</Button>
        <Button :size="ButtonSize.Tall" :icon="Share">Esporta…</Button>
        <Button :variant="ButtonVariant.GhostRegular" :size="ButtonSize.Small" :icon="Undo2">
          Ripristina
        </Button>
      </div>
    </KitState>
    <KitState label="icon + text · Danger, and disabled (the whole button fades)">
      <div class="flex flex-wrap items-center gap-8">
        <Button :variant="ButtonVariant.Danger" :size="ButtonSize.Medium" :icon="Trash2">
          Elimina lo slot 2
        </Button>
        <Button :size="ButtonSize.Medium" :icon="Save" disabled>Salva</Button>
      </div>
    </KitState>
    <KitState label="icon only (label = aria-label + tooltip) · IconRow, IconSmall, Icon, ToolIcon">
      <div class="flex flex-wrap items-center gap-8">
        <Button :size="ButtonSize.IconRow" :icon="Play" icon-filled label="Gioca questa" />
        <Button
          :variant="ButtonVariant.GhostRegular"
          :size="ButtonSize.IconSmall"
          :icon="Pin"
          label="Mostra questa in libreria"
        />
        <Button :size="ButtonSize.Icon" :icon="Share" label="Esporta…" />
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.ToolIcon"
          :icon="Play"
          icon-filled
          label="Riprendi"
        />
      </div>
    </KitState>
    <KitState label="icon only · disabled">
      <Button :size="ButtonSize.IconRow" :icon="Trash2" label="Togli" disabled />
    </KitState>
    <KitState
      label="On the phone (touch density): icons one step larger; IconRow 44, icon + text Default and Large"
    >
      <div data-density="touch" class="flex flex-wrap items-center gap-8">
        <Button :size="ButtonSize.IconRow" :icon="Play" icon-filled label="Gioca questa" />
        <Button :icon="FolderPlus">Aggiungi una cartella…</Button>
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Large" :icon="Play" icon-filled>
          Riprendi
        </Button>
      </div>
    </KitState>
    <KitState
      label="RowAction: the phone list's «Riprendi» / «Gioca», 32 drawn, 44 to the touch (the dashed box)"
    >
      <div
        data-density="touch"
        class="flex w-page max-w-full items-center gap-12 rounded-card border border-line bg-surface-raised py-8 pr-8 pl-control-pad"
      >
        <div class="flex min-w-0 flex-1 flex-col gap-2">
          <span class="truncate font-semibold">Pokémon Smeraldo</span>
          <span class="truncate text-caption text-ink-muted">GBA · ieri</span>
        </div>
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.RowAction"
          :icon="Play"
          icon-filled
          label="Riprendi"
          class="before:outline-ring before:outline-ink-faint before:outline-dashed"
        />
        <Button :size="ButtonSize.RowAction" :icon="Play" icon-filled label="Gioca" />
      </div>
    </KitState>
  </KitSection>
</template>
