<script setup lang="ts">
// UnreachableBanner (library/UnreachableBanner.vue): E01, a library folder that is not reachable,
// or a picked Android folder whose read permission Android took back (#138).
import UnreachableBanner from '@/components/library/UnreachableBanner.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'

const noop = () => {}
const NAS = '//nas-salotto/condivisa/Emulazione/Nintendo DS/Collezione completa (Europa)'
const TREE = 'content://com.android.externalstorage.documents/tree/primary%3AROMs'
</script>

<template>
  <KitSection title="Unreachable folder banner" :group="KitGroup.Library">
    <KitState label="Short path, one game">
      <div class="w-page-md max-w-full">
        <UnreachableBanner
          :folder="{ path: 'E:/Giochi', name: 'E:/Giochi', count: 1, access: 'unreachable' }"
          @replace="noop"
        />
      </div>
    </KitState>
    <KitState label="Long network path, 312 games">
      <div class="w-page-md max-w-full">
        <UnreachableBanner
          :folder="{ path: NAS, name: NAS, count: 312, access: 'unreachable' }"
          @replace="noop"
        />
      </div>
    </KitState>
    <KitState label="Narrow (tile-min)">
      <div class="w-tile-min">
        <UnreachableBanner
          :folder="{
            path: 'E:/Giochi/GBA',
            name: 'E:/Giochi/GBA',
            count: 4,
            access: 'unreachable',
          }"
          @replace="noop"
        />
      </div>
    </KitState>
    <KitState label="Android: permission taken back (#138)">
      <div class="w-tile-min">
        <UnreachableBanner
          :folder="{ path: TREE, name: 'ROMs', count: 24, access: 'permissionLost' }"
          @replace="noop"
        />
      </div>
    </KitState>
  </KitSection>
</template>
