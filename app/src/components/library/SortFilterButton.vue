<script setup lang="ts">
// «Ordina per: …» over the grid, which opens the L03 panel (a Popover) under it; closing the
// panel (B, Esc, a click outside, the button again) puts the focus back on the button.
// `short` (the phone's toolbar, #29): the order's name alone («Ultima partita»), «Ordina per: …»
// kept as the button's accessible name.
import { computed, nextTick, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronDown } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Popover, PopoverContent } from '@/components/ui/popover'
import type { ShowKey, Sort } from './librarySort'
import LibraryFilters from './LibraryFilters.vue'

withDefaults(defineProps<{ sort: Sort; show: Record<ShowKey, boolean>; short?: boolean }>(), {
  short: false,
})
const emit = defineEmits<{ sort: [Sort]; toggle: [ShowKey]; reset: [] }>()

const { t } = useI18n()

const open = ref(false)
const button = ref<InstanceType<typeof Button> | null>(null)
const buttonEl = computed(() => (button.value?.$el as HTMLElement | undefined) ?? null)

function close() {
  open.value = false
  void nextTick(() => buttonEl.value?.focus())
}
</script>

<template>
  <!-- As wide as the caller makes it (the phone's toolbar: the row's room), the words cut short
       before the chevron when they do not fit. -->
  <div class="relative flex min-w-0">
    <Popover :open="open" @update:open="o => o || close()">
      <Button
        ref="button"
        :variant="ButtonVariant.Quiet"
        :size="ButtonSize.Tool"
        class="min-w-0 flex-1 justify-between"
        :aria-label="short ? t(`library.sortButton.${sort}`) : undefined"
        :aria-expanded="open"
        aria-haspopup="dialog"
        @click="open ? close() : (open = true)"
      >
        <span class="min-w-0 truncate">{{
          t(short ? `library.sort.${sort}` : `library.sortButton.${sort}`)
        }}</span>
        <ChevronDown :size="14" :stroke-width="1.75" aria-hidden="true" />
      </Button>
      <PopoverContent :anchor="buttonEl" :aria-label="t('library.filters.label')">
        <LibraryFilters
          :sort="sort"
          :show="show"
          @sort="s => emit('sort', s)"
          @toggle="k => emit('toggle', k)"
          @reset="emit('reset')"
        />
      </PopoverContent>
    </Popover>
  </div>
</template>
