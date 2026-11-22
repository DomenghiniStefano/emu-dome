<script setup lang="ts">
import { ref } from 'vue'
import { Search } from '@lucide/vue'
import { Input, InputVariant } from '@/components/ui/input'
import { SearchField } from '@/components/ui/search-field'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'

const empty = ref('')
const filled = ref('Pocket Quest')
const saves = ref('')
const settings = ref('')
</script>

<template>
  <KitSection title="SearchField">
    <KitState label="Default · empty">
      <SearchField
        v-model="empty"
        class="w-field"
        label="Search for a game"
        placeholder="Search for a game"
      />
    </KitState>
    <KitState label="Default · with a query">
      <SearchField
        v-model="filled"
        class="w-field"
        label="Search for a game"
        placeholder="Search for a game"
      />
    </KitState>
    <template #review>
      <KitIssue title="Due caselle di ricerca diverse" :status="IssueStatus.Proposed">
        Salvataggi usa SearchField (alta 28, raggio 2, lente da 14). La ricerca delle Impostazioni
        (SettingsCategories.vue) è costruita a mano con Input Bare: alta 40, raggio 8, lente da 16,
        margine 12. Sono la stessa cosa in due sezioni vicine della shell, con due misure.
        <template #current>
          <div class="flex flex-col gap-8">
            <SearchField
              v-model="saves"
              class="w-sidebar"
              label="Search, Saves"
              placeholder="Search for a game"
            />
            <label
              class="flex h-control-lg w-sidebar items-center gap-8 rounded-8 border border-line-strong bg-surface-raised px-12 text-ink-muted"
            >
              <Search :size="16" :stroke-width="1.75" aria-hidden="true" />
              <span class="sr-only">Search, Settings</span>
              <Input
                v-model="settings"
                :variant="InputVariant.Bare"
                type="search"
                placeholder="Search settings"
              />
            </label>
          </div>
        </template>
        <template #proposed>
          <div class="flex flex-col gap-8">
            <SearchField
              v-model="saves"
              class="w-sidebar"
              label="Search, Saves (proposed)"
              placeholder="Search for a game"
            />
            <SearchField
              v-model="settings"
              class="w-sidebar"
              label="Search, Settings (proposed)"
              placeholder="Search settings"
            />
          </div>
        </template>
        <template #solution>
          SettingsCategories.vue usa SearchField; se le Impostazioni devono restare alte 40, una
          taglia Large in search-field/variants.ts (SearchFieldSize ha oggi solo Default).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
