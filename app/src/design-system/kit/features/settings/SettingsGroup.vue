<script setup lang="ts">
// The feature components of this area on the Kit page, one section file each in this folder
// (KitSection with group KitGroup.Settings), listed here in the order they show. The setting
// rows reach the app's toasts (useToasts): the group gives them a list of their own, as App.vue
// does, which nothing shows.
import { createToasts, provideToasts } from '@/composables/useToasts'
import SaveConflictSection from './SaveConflictSection.vue'
import SavesEmptySection from './SavesEmptySection.vue'
import SavesGameRowSection from './SavesGameRowSection.vue'
import SaveWarningSection from './SaveWarningSection.vue'
import SettingRowBlocksSection from './SettingRowBlocksSection.vue'
import SettingRowControlsSection from './SettingRowControlsSection.vue'
import SettingRowStatesSection from './SettingRowStatesSection.vue'
import SettingsScopeBarSection from './SettingsScopeBarSection.vue'
import SlotTileSection from './SlotTileSection.vue'

provideToasts(createToasts())
</script>

<template>
  <div class="flex flex-col gap-24">
    <SettingRowStatesSection />
    <SettingRowControlsSection />
    <SettingRowBlocksSection />
    <SettingsScopeBarSection />
    <SlotTileSection />
    <SavesGameRowSection />
    <SaveWarningSection />
    <SaveConflictSection />
    <SavesEmptySection />
  </div>
</template>
