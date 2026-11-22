<script setup lang="ts">
// StatTile: the games per console, in the first run's S02 (four tiles, zeros included, two
// columns on the phone) and in the library's L11 after a scan (auto-filled, zeros hidden).
import { StatTile } from '@/components/ui/stat-tile'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import StatTileZeroProposal from '@/kit/proposals/StatTileZeroProposal.vue'
import { IssueStatus } from '@/kit/review'

const FIRST_RUN = [
  { id: 'gba', label: 'Game Boy Advance', count: 42 },
  { id: 'nds', label: 'Nintendo DS', count: 7 },
  { id: '3ds', label: 'Nintendo 3DS', count: 0 },
  { id: 'gb', label: 'Game Boy e Color', count: 0 },
]
const BIG = [
  { id: 'gba', label: 'Game Boy Advance', count: 1234 },
  { id: 'nds', label: 'Nintendo DS', count: 12 },
]
const it = new Intl.NumberFormat('it')
</script>

<template>
  <KitSection title="StatTile">
    <KitState label="First run S02: four tiles, zeros included">
      <div class="grid w-panel grid-cols-4 gap-6">
        <StatTile v-for="c in FIRST_RUN" :key="c.id" :value="c.count" :label="c.label" />
      </div>
    </KitState>
    <KitState label="Library L11 after a scan: auto-fill grid, zeros hidden">
      <div
        class="grid w-panel grid-cols-[repeat(auto-fill,minmax(var(--spacing-stat-min),1fr))] gap-6"
      >
        <StatTile
          v-for="c in FIRST_RUN.filter(x => x.count > 0)"
          :key="c.id"
          :value="c.count"
          :label="c.label"
        />
      </div>
    </KitState>
    <KitState label="On the phone (touch density, two columns)">
      <div data-density="touch" class="grid w-field grid-cols-2 gap-10">
        <StatTile v-for="c in FIRST_RUN" :key="c.id" :value="c.count" :label="c.label" />
      </div>
    </KitState>
    <KitState label="A four-digit count, and a string value">
      <div class="grid w-dialog grid-cols-3 gap-6">
        <StatTile v-for="c in BIG" :key="c.id" :value="c.count" :label="c.label" />
        <StatTile value="—" label="Nintendo 3DS" />
      </div>
    </KitState>

    <template #review>
      <KitIssue title="Lo zero pesa come un conteggio vero" :status="IssueStatus.Proposed">
        Nel primo avvio (S02) le quattro caselle ci sono sempre, anche a zero: «0» è grande e bianco
        come «42», e a colpo d'occhio sembra che ogni console abbia giochi. Durante la ricerca i
        numeri crescono, ma una console senza giochi non si distingue da una che ne ha.
        <template #current>
          <div class="grid w-panel grid-cols-4 gap-6">
            <StatTile v-for="c in FIRST_RUN" :key="c.id" :value="c.count" :label="c.label" />
          </div>
        </template>
        <template #proposed>
          <div class="grid w-panel grid-cols-4 gap-6">
            <StatTileZeroProposal
              v-for="c in FIRST_RUN"
              :key="c.id"
              :value="c.count"
              :label="c.label"
            />
          </div>
        </template>
        <template #solution>
          in components/ui/stat-tile/StatTile.vue il numero in `text-ink-faint` quando vale 0. Le
          caselle restano quattro (la griglia non salta durante la ricerca).
        </template>
      </KitIssue>

      <KitIssue title="Numeri senza separatore delle migliaia" :status="IssueStatus.Proposed">
        Il numero è stampato così com'è: 1234 giochi si leggono «1234», non «1.234» in italiano né
        «1,234» in inglese. Con una libreria grande (chi ha tutto il set GBA) il numero più
        importante della schermata è anche il meno leggibile.
        <template #current>
          <div class="grid w-menu grid-cols-2 gap-6">
            <StatTile v-for="c in BIG" :key="c.id" :value="c.count" :label="c.label" />
          </div>
        </template>
        <template #proposed>
          <div class="grid w-menu grid-cols-2 gap-6">
            <StatTile v-for="c in BIG" :key="c.id" :value="it.format(c.count)" :label="c.label" />
          </div>
        </template>
        <template #solution>
          StatTile.vue formatta un `value` numerico con `n()` di vue-i18n (la lingua dell'app), così
          GamesStep e ScanSummary non cambiano. Il valore stringa resta com'è.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
