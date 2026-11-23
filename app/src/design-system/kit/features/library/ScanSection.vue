<script setup lang="ts">
// The scan's two banners over the grid: ScanProgress while it runs, ScanSummary (L11) once done;
// with UnreachableBanner they share one shape (library/LibraryBanner.vue).
import ScanProgress from '@/components/library/ScanProgress.vue'
import ScanSummary from '@/components/library/ScanSummary.vue'
import UnreachableBanner from '@/components/library/UnreachableBanner.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'

const noop = () => {}
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
        <ScanSummary :summary="{ added: 1284, perConsole: EVERY }" @close="noop" />
      </div>
    </KitState>
    <KitState label="All three over the grid, narrow (tile-min)">
      <div class="flex w-tile-min flex-col gap-8">
        <ScanProgress :progress="{ found: 1284 }" />
        <ScanSummary :summary="{ added: 1284, perConsole: EVERY }" @close="noop" />
        <UnreachableBanner
          :folder="{ path: 'E:/Giochi', name: 'E:/Giochi', count: 1 }"
          @replace="noop"
        />
      </div>
    </KitState>
  </KitSection>
</template>
