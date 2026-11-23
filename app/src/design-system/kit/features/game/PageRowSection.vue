<script setup lang="ts">
// The quick menu's rows to another page (G07 «Schermi e immagine», «Controlli»; G08 the backups):
// icon, name, chevron.
import { ArchiveRestore, Gamepad2, Rows2 } from '@lucide/vue'
import PageRow from '@/components/quick-menu/PageRow.vue'
import { LinkRow } from '@/design-system/components/link-row'
import { Card, CardVariant } from '@/design-system/components/card'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/design-system/kit/review'
import GameStage from './GameStage.vue'
</script>

<template>
  <KitSection title="Page row" :group="KitGroup.Game">
    <KitState label="Two side by side (G07), and disabled (G08 backups)">
      <GameStage panel>
        <div class="grid grid-cols-[1fr_1fr] gap-8">
          <PageRow :icon="Rows2" label="Schermi e immagine" />
          <PageRow :icon="Gamepad2" label="Controlli" />
        </div>
        <PageRow :icon="ArchiveRestore" label="Backup del salvataggio del gioco" disabled />
      </GameStage>
    </KitState>
    <KitState label="A long name in a half-width column">
      <GameStage panel>
        <div class="grid grid-cols-[1fr_1fr] gap-8">
          <PageRow :icon="Rows2" label="Schermi, immagine ed effetti dello schermo" />
          <PageRow :icon="Gamepad2" label="Controlli" />
        </div>
      </GameStage>
    </KitState>

    <template #review>
      <KitIssue
        title="Due righe «vai a un’altra pagina» con due aspetti"
        :status="IssueStatus.Open"
      >
        PageRow (menu rapido) e la primitiva LinkRow (Impostazioni, Controlli) fanno la stessa cosa,
        portano a un’altra pagina con un chevron, ma hanno bordo, riempimento, icona e altezza
        diversi. Il commento della variante <code>CardVariant.NavRow</code> lo dice già («Not
        ui/link-row: its own edge and fill»). Sotto, a sinistra la riga del menu, a destra quella di
        LinkRow.
        <template #current>
          <div class="w-tile-min max-w-full">
            <PageRow :icon="Gamepad2" label="Controlli" />
          </div>
          <Card :variant="CardVariant.List" class="w-tile-min max-w-full">
            <LinkRow name="Controlli" sub="Pulsanti e scorciatoie" />
          </Card>
        </template>
        <template #solution>
          da decidere: o PageRow diventa una variante di LinkRow (una riga staccata, con icona, per
          il vetro del menu), o si scrive nel design system che dentro il vetro le righe sono card e
          sulle pagine sono righe di lista. In ogni caso un solo posto per il chevron e l’altezza.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
