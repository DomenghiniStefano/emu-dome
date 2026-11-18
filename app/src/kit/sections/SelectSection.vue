<script setup lang="ts">
import { ref } from 'vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const options = [
  { value: 'system', label: 'Like the system' },
  { value: 'en', label: 'English' },
  { value: 'it', label: 'Italiano' },
  { value: 'later', label: 'Not yet', disabled: true },
]
const language = ref('system')
</script>

<template>
  <KitSection title="Select">
    <KitState label="Closed (A / Enter opens; B, Escape or a click outside close)">
      <Select v-model="language">
        <SelectTrigger aria-label="Language"><SelectValue /></SelectTrigger>
        <SelectContent>
          <SelectItem v-for="o in options" :key="o.value" :value="o.value" :disabled="o.disabled">
            {{ o.label }}
          </SelectItem>
        </SelectContent>
      </Select>
    </KitState>
    <KitState label="Disabled">
      <Select model-value="en" disabled>
        <SelectTrigger aria-label="Language"><SelectValue /></SelectTrigger>
        <SelectContent>
          <SelectItem v-for="o in options" :key="o.value" :value="o.value">{{
            o.label
          }}</SelectItem>
        </SelectContent>
      </Select>
    </KitState>
  </KitSection>
</template>
