<script setup lang="ts">
// The feature components of this area on the Kit page, one section file each in this folder
// (KitSection with group KitGroup.Game), listed here in the order they show: the quick menu's
// parts, what shows over the game, the TV shell's bars, then Controls.
import DeviceCardSection from './DeviceCardSection.vue'
import FlowDialogSection from './FlowDialogSection.vue'
import HudChipSection from './HudChipSection.vue'
import MenuFooterSection from './MenuFooterSection.vue'
import PageRowSection from './PageRowSection.vue'
import QuickSavesSection from './QuickSavesSection.vue'
import RemapTableRowSection from './RemapTableRowSection.vue'
import ScreenCardSection from './ScreenCardSection.vue'
import ShortcutRowSection from './ShortcutRowSection.vue'
import SpeedPickerSection from './SpeedPickerSection.vue'
import TesterCardSection from './TesterCardSection.vue'
import ToastStackSection from './ToastStackSection.vue'
import ToolTilesSection from './ToolTilesSection.vue'
import TrialPanelSection from './TrialPanelSection.vue'
import TvBarsSection from './TvBarsSection.vue'
import TvGlyphSection from './TvGlyphSection.vue'
import UndoLoadBannerSection from './UndoLoadBannerSection.vue'
</script>

<template>
  <div class="flex flex-col gap-24">
    <QuickSavesSection />
    <SpeedPickerSection />
    <ToolTilesSection />
    <PageRowSection />
    <ScreenCardSection />
    <TrialPanelSection />
    <UndoLoadBannerSection />
    <MenuFooterSection />
    <HudChipSection />
    <ToastStackSection />
    <FlowDialogSection />
    <TvGlyphSection />
    <TvBarsSection />
    <DeviceCardSection />
    <ShortcutRowSection />
    <RemapTableRowSection />
    <TesterCardSection />
  </div>
</template>
