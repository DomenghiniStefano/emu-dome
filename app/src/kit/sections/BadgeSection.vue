<script setup lang="ts">
import { Badge, BadgeSize, BadgeVariant } from '@/components/ui/badge'
import { Kbd, KbdSize, KbdVariant } from '@/components/ui/kbd'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
</script>

<template>
  <KitSection title="Badge">
    <KitState label="Tag: Neutral, Warning, Reserved">
      <div class="flex items-center gap-8">
        <Badge>Newest</Badge>
        <Badge :variant="BadgeVariant.Warning">Save unreadable</Badge>
        <Badge :variant="BadgeVariant.Reserved">Coming soon</Badge>
      </div>
    </KitState>
    <KitState label="Pill: Success, Quiet">
      <div class="flex items-center gap-8">
        <Badge :variant="BadgeVariant.Success" :size="BadgeSize.Pill">In use</Badge>
        <Badge :variant="BadgeVariant.Quiet" :size="BadgeSize.Pill">crt.slangp</Badge>
      </div>
    </KitState>
  </KitSection>
  <KitSection title="Kbd">
    <KitState label="Default: in a line of text, in a list">
      <span class="inline-flex items-center gap-6 text-ink-muted">
        <Kbd>Esc</Kbd> or <Kbd>Home</Kbd> for the quick menu
      </span>
    </KitState>
    <KitState label="Small: beside an action's label, on a surface and on the accent">
      <div class="flex items-center gap-8">
        <span class="inline-flex items-center rounded-control bg-surface-raised px-12 py-6">
          Back <Kbd :size="KbdSize.Small" class="ml-4">B</Kbd>
        </span>
        <span
          class="inline-flex items-center rounded-control bg-accent-dome px-12 py-6 text-on-accent"
        >
          Resume <Kbd :variant="KbdVariant.OnAccent" :size="KbdSize.Small" class="ml-4">Esc</Kbd>
        </span>
      </div>
    </KitState>
  </KitSection>
</template>
