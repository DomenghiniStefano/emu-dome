<script setup lang="ts">
import { Save } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { EmptyState, EmptyStateVariant } from '@/components/ui/empty-state'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'
</script>

<template>
  <KitSection title="EmptyState">
    <KitState label="Page: icon, title, text, actions, footer">
      <div class="flex w-menu max-w-full border border-dashed border-line">
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
        class="w-menu max-w-full"
        text="No setting matches “crt”."
      />
    </KitState>
    <template #review>
      <KitIssue
        title="ink-faint è sotto il contrasto minimo per il testo"
        :status="IssueStatus.Proposed"
      >
        ink-faint contro lo sfondo fa 3,8:1 in Scuro e 2,5:1 in Chiaro; il design system chiede
        4,5:1 per il testo. In app lo usa testo che si deve leggere: il piede di SavesEmpty («.sav,
        .srm, .dsv», stato Page qui sopra) e la riga «da …» di ContinueCard.
        <template #current>
          <p class="m-0 text-caption text-ink-faint">.sav, .srm, .dsv</p>
        </template>
        <template #proposed>
          <p class="m-0 text-caption text-ink-muted">.sav, .srm, .dsv</p>
        </template>
        <template #solution>
          Il testo da leggere usa ink-muted (SavesEmpty.vue, ContinueCard.vue); ink-faint resta per
          ciò che è disabilitato o decorativo, scritto in docs/frontend-conventions.md. Oppure
          ink-faint si sposta fino a 4,5:1 in colors.css, ma allora si avvicina a ink-muted.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
