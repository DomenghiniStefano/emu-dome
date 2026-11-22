<script setup lang="ts">
// The scan's two banners over the grid: ScanProgress while it runs, ScanSummary (L11) once done.
import ScanProgress from '@/components/library/ScanProgress.vue'
import ScanSummary from '@/components/library/ScanSummary.vue'
import UnreachableBanner from '@/components/library/UnreachableBanner.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import LibraryBannerProposal from '@/kit/proposals/LibraryBannerProposal.vue'
import { IssueStatus, KitGroup } from '@/kit/review'

const noop = () => {}
const ONE = { gba: 1 }
const EVERY = { gb: 211, gbc: 187, gba: 612, nds: 251, '3ds': 23 }
</script>

<template>
  <KitSection title="Scan progress and summary" :group="KitGroup.Library">
    <KitState label="Progress · starting (no count yet)">
      <div class="w-page-md max-w-full"><ScanProgress :progress="null" /></div>
    </KitState>
    <KitState label="Progress · none found yet">
      <div class="w-page-md max-w-full"><ScanProgress :progress="{ found: 0 }" /></div>
    </KitState>
    <KitState label="Progress · 1284 found">
      <div class="w-page-md max-w-full"><ScanProgress :progress="{ found: 1284 }" /></div>
    </KitState>
    <KitState label="Summary · one console">
      <div class="w-page-md max-w-full">
        <ScanSummary :summary="{ added: 1, perConsole: { gba: 1 } }" @close="noop" />
      </div>
    </KitState>
    <KitState label="Summary · every console">
      <div class="w-page-md max-w-full">
        <ScanSummary
          :summary="{
            added: 1284,
            perConsole: { gb: 211, gbc: 187, gba: 612, nds: 251, '3ds': 23 },
          }"
          @close="noop"
        />
      </div>
    </KitState>
    <template #review>
      <KitIssue
        title="Tre avvisi sopra la griglia, tre aspetti diversi"
        :status="IssueStatus.Proposed"
      >
        Sopra la griglia possono comparire insieme la ricerca in corso (una riga nuda, senza bordo
        né fondo), «N giochi aggiunti» (un pannello glass con padding 20, titolo Heading e riquadri
        da 160 px) e la cartella non raggiungibile (una riga con bordo e fondo warning). Tre forme
        per lo stesso ruolo. Il riepilogo in più è il blocco più alto della pagina per dire un
        numero: con una sola console resta un riquadro a sinistra e una fascia vuota a destra.
        <template #current>
          <div class="flex w-full flex-col gap-12">
            <ScanProgress :progress="{ found: 1284 }" />
            <ScanSummary :summary="{ added: 1, perConsole: ONE }" @close="noop" />
            <UnreachableBanner :folder="{ path: 'E:/Giochi', count: 1 }" @replace="noop" />
          </div>
        </template>
        <template #proposed>
          <div class="flex w-full flex-col gap-12">
            <LibraryBannerProposal kind="progress" :found="1284" />
            <LibraryBannerProposal kind="summary" :added="1" :per-console="ONE" />
            <LibraryBannerProposal kind="summary" :added="1284" :per-console="EVERY" />
            <LibraryBannerProposal kind="unreachable" path="E:/Giochi" :count="1" />
          </div>
        </template>
        <template #solution>
          un solo componente per gli avvisi della libreria (per esempio
          <code>library/LibraryBanner.vue</code>, o una variante di un primitivo
          <code>Banner</code> in <code>components/ui</code> con tono Neutral / Warning): bordo,
          raggio 10, padding 14 × 10, a capo quando serve. ScanProgress, ScanSummary e
          UnreachableBanner ci mettono solo il contenuto; il riepilogo diventa una riga con i
          conteggi in fila e la X piccola.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
