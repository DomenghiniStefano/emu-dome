<script setup lang="ts">
// TV mode's two bars (#30): the sections as tabs at the top (TvTabs, L1 / R1 at their sides) and
// the button bar at the bottom (TvButtonBar: what the controller's buttons do in the section).
// Drawn for 1920 × 1080, so they are wider than most of the page: the box scrolls sideways.
import TvButtonBar from '@/components/shell/TvButtonBar.vue'
import TvTabs from '@/components/shell/TvTabs.vue'
import { tvHints } from '@/components/shell/tvHints'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { KitGroup } from '@/kit/review'
</script>

<template>
  <KitSection title="TV tabs and button bar" :group="KitGroup.Game">
    <KitState label="Tabs: Libreria current">
      <div class="flex w-page-wide max-w-full flex-col overflow-x-auto rounded-card bg-bg">
        <TvTabs current="library" content-has-initial />
      </div>
    </KitState>
    <KitState label="Button bar: the library, the settings, a game's details">
      <div class="flex w-page-wide max-w-full flex-col overflow-x-auto rounded-card bg-bg">
        <TvButtonBar :hints="tvHints('library')" />
        <TvButtonBar :hints="tvHints('settings')" />
        <TvButtonBar :hints="tvHints('library', true)" />
      </div>
    </KitState>
  </KitSection>
</template>
