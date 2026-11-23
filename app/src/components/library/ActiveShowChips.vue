<script setup lang="ts">
// The L03 «Mostra» switches that are on, as chips on the phone's toolbar (#132): on the phone the
// panel is out of sight behind the order's button, so a switch left on (Settings' «Mostra i
// giochi nascosti» turns «Anche i giochi nascosti» on) is said here, in L03's own words, and a tap
// on its chip turns it off. A chip is a pressed toggle (`aria-pressed`): pressing it lets go.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { X } from '@lucide/vue'
import { Chip, ChipVariant } from '@/design-system/components/chip'
import { SHOW_KEYS, type ShowKey } from './librarySort'

const props = defineProps<{ show: Record<ShowKey, boolean> }>()
const emit = defineEmits<{ toggle: [ShowKey] }>()
const { t } = useI18n()

const on = computed(() => SHOW_KEYS.filter(k => props.show[k]))
</script>

<template>
  <Chip
    v-for="k in on"
    :key="k"
    :variant="ChipVariant.Filter"
    selected
    aria-pressed="true"
    @click="emit('toggle', k)"
  >
    {{ t(`library.show.${k}`) }}
    <X :size="14" :stroke-width="1.75" aria-hidden="true" />
  </Chip>
</template>
