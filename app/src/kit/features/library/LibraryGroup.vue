<script setup lang="ts">
// The feature components of this area on the Kit page, one section file each in this folder
// (KitSection with group KitGroup.Library), listed here in the order they show.
import ConsoleChipsSection from './ConsoleChipsSection.vue'
import ContinueCardSection from './ContinueCardSection.vue'
import CoverChoicesSection from './CoverChoicesSection.vue'
import CoverTileSection from './CoverTileSection.vue'
import DetailsHeroSection from './DetailsHeroSection.vue'
import GameCoverSection from './GameCoverSection.vue'
import GameFileCardSection from './GameFileCardSection.vue'
import LibraryEmptySection from './LibraryEmptySection.vue'
import OverrideRowSection from './OverrideRowSection.vue'
import ScanSection from './ScanSection.vue'
import UnreachableBannerSection from './UnreachableBannerSection.vue'
</script>

<template>
  <div class="flex flex-col gap-16">
    <GameCoverSection />
    <CoverTileSection />
    <ContinueCardSection />
    <ConsoleChipsSection />
    <ScanSection />
    <UnreachableBannerSection />
    <LibraryEmptySection />
    <DetailsHeroSection />
    <OverrideRowSection />
    <GameFileCardSection />
    <CoverChoicesSection />
  </div>
</template>
