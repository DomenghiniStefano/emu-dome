<script setup lang="ts">
// «Ordina per: …» over the grid, which opens the L03 panel under it; closing the panel puts the
// focus back on the button.
import { nextTick, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronDown } from '@lucide/vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import type { ShowKey, Sort } from './librarySort'
import LibraryFilters from './LibraryFilters.vue'

defineProps<{ sort: Sort; show: Record<ShowKey, boolean> }>()
const emit = defineEmits<{ sort: [Sort]; toggle: [ShowKey]; reset: [] }>()

const { t } = useI18n()

const open = ref(false)
const button = ref<InstanceType<typeof Button> | null>(null)

function close() {
  open.value = false
  void nextTick(() => (button.value?.$el as HTMLElement | undefined)?.focus())
}
</script>

<template>
  <div class="relative">
    <Button
      ref="button"
      :variant="ButtonVariant.Quiet"
      :aria-expanded="open"
      aria-haspopup="dialog"
      @click="open = !open"
    >
      {{ t(`library.sortButton.${sort}`) }}
      <ChevronDown :size="14" :stroke-width="1.75" aria-hidden="true" />
    </Button>
    <LibraryFilters
      v-if="open"
      :sort="sort"
      :show="show"
      @sort="s => emit('sort', s)"
      @toggle="k => emit('toggle', k)"
      @reset="emit('reset')"
      @close="close"
    />
  </div>
</template>
