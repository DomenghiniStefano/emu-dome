<script setup lang="ts">
import { ref } from 'vue'
import { Search } from '@lucide/vue'
import { Input, InputVariant } from '@/components/ui/input'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const server = ref('')
const query = ref('')
const empty = ref('')
const filled = ref('Pocket Quest')
</script>

<template>
  <KitSection title="Input">
    <KitState label="Field, with a placeholder">
      <Input
        v-model="server"
        :variant="InputVariant.Field"
        aria-label="Server"
        placeholder="None"
      />
    </KitState>
    <KitState label="Field · disabled">
      <Input :variant="InputVariant.Field" model-value="wfc.example" aria-label="Server" disabled />
    </KitState>
    <KitState label="Block · empty, with a placeholder">
      <Input
        v-model="empty"
        :variant="InputVariant.Block"
        class="w-field"
        placeholder="Pocket Quest Advance"
      />
    </KitState>
    <KitState label="Block · filled">
      <Input v-model="filled" :variant="InputVariant.Block" class="w-field" />
    </KitState>
    <KitState label="Block · disabled">
      <Input v-model="filled" :variant="InputVariant.Block" class="w-field" disabled />
    </KitState>
    <KitState label="Bare, inside a box its container draws (a search)">
      <label
        class="flex h-control-lg w-sidebar items-center gap-8 rounded-6 border border-line-strong bg-surface-raised px-12 text-ink-muted"
      >
        <Search :size="16" :stroke-width="1.75" aria-hidden="true" />
        <Input v-model="query" :variant="InputVariant.Bare" type="search" placeholder="Search" />
      </label>
    </KitState>
  </KitSection>
</template>
