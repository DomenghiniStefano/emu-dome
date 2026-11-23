<script setup lang="ts">
// UnreachableBanner (library/UnreachableBanner.vue): E01, a library folder that is not reachable.
import UnreachableBanner from '@/components/library/UnreachableBanner.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'

const noop = () => {}
const NAS = '//nas-salotto/condivisa/Emulazione/Nintendo DS/Collezione completa (Europa)'
</script>

<template>
  <KitSection title="Unreachable folder banner" :group="KitGroup.Library">
    <KitState label="Short path, one game">
      <div class="w-page-md max-w-full">
        <UnreachableBanner
          :folder="{ path: 'E:/Giochi', name: 'E:/Giochi', count: 1 }"
          @replace="noop"
        />
      </div>
    </KitState>
    <KitState label="Long network path, 312 games">
      <div class="w-page-md max-w-full">
        <UnreachableBanner :folder="{ path: NAS, name: NAS, count: 312 }" @replace="noop" />
      </div>
    </KitState>
    <KitState label="Narrow (tile-min)">
      <div class="w-tile-min">
        <UnreachableBanner
          :folder="{ path: 'E:/Giochi/GBA', name: 'E:/Giochi/GBA', count: 4 }"
          @replace="noop"
        />
      </div>
    </KitState>
  </KitSection>
</template>
