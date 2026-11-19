<script setup lang="ts">
import { Save } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { EmptyState, EmptyStateVariant } from '@/components/ui/empty-state'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
</script>

<template>
  <KitSection title="EmptyState">
    <KitState label="Page: icon, title, text, actions, footer">
      <div class="flex h-dialog w-dialog border border-dashed border-line">
        <EmptyState
          :icon="Save"
          title="Nothing saved yet"
          text="Your progress saves itself when you close a game."
        >
          <Button :size="ButtonSize.Medium">Open the library</Button>
          <Button :size="ButtonSize.Medium">Import saves…</Button>
          <template #footer>
            <p class="m-0 text-caption text-ink-faint">.sav, .srm, .dsv</p>
          </template>
        </EmptyState>
      </div>
    </KitState>
    <KitState label="Framed: a box that also takes a drop">
      <EmptyState
        :variant="EmptyStateVariant.Framed"
        :icon="Save"
        title="Add your games"
        text="Choose a folder, or drag games here."
      >
        <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Medium">Choose a folder</Button>
      </EmptyState>
    </KitState>
    <KitState label="Card: inside a list, text only">
      <EmptyState
        as="div"
        :variant="EmptyStateVariant.Card"
        class="w-dialog"
        text="No setting matches “crt”."
      />
    </KitState>
  </KitSection>
</template>
